//! Local yield pre-estimate and orientation sweep. `preestimar` ranks orientations
//! with the model in `sites::solar`, so it costs no trial execution. `barrido` sends
//! the best orientations of that sweep to one batch simulation, after the trial
//! budget confirmation check in `ledger`.

use super::batch;
use super::lanzar_simulacion_seguida;
use crate::adapters::ledger;
use crate::args::{opt_str, req_str, workspace_of};
use crate::config::Paths;
use crate::error::{Error, Result};
use crate::progreso::SeguimientoTarea;
use crate::sites::solar::{
    numero_argumento, orientacion_de, produccion, redondear, requerido, sitio_de, Clima,
    Orientacion, Produccion, Sitio, ALBEDO, GAMMA_POTENCIA, NOCT, PERDIDAS_SISTEMA,
};
use serde_json::{json, Value};

/// Orientations ranked in a `preestimar` sweep and default count for `barrido`.
const TOP_PREESTIMACION: usize = 5;
const N_BARRIDO_DEFECTO: usize = 3;

/// Sweep grid: tilt 0..=60 in steps of 5 and azimuth -90..=90 in steps of 15 (degrees).
fn rejilla() -> Vec<Orientacion> {
    (0..=60_i32)
        .step_by(5)
        .flat_map(|t| {
            (-90..=90_i32).step_by(15).map(move |a| Orientacion {
                inclinacion: f64::from(t),
                azimut: f64::from(a),
            })
        })
        .collect()
}

/// Every orientation of the sweep grid with its production, best specific yield first.
fn ranking(sitio: &Sitio, clima: &Clima, kwp: f64, n: usize) -> Vec<(Orientacion, Produccion)> {
    let mut todas: Vec<(Orientacion, Produccion)> = rejilla()
        .into_iter()
        .map(|o| {
            let p = produccion(sitio, clima, o, kwp);
            (o, p)
        })
        .collect();
    todas.sort_by(|a, b| b.1.especifica.total_cmp(&a.1.especifica));
    todas.truncate(n);
    todas
}

/// Positive plant size in kWp from the `kwp` argument.
fn kwp_de(args: &Value) -> Result<f64> {
    let kwp = requerido(args, "kwp")?;
    if kwp > 0.0 {
        Ok(kwp)
    } else {
        Err(Error::Invalid("'kwp' debe ser mayor que 0".into()))
    }
}

/// Number of top orientations for `barrido`: an integer from 1 to the grid size.
fn numero_de_orientaciones(args: &Value) -> Result<usize> {
    let n = numero_argumento(args, "n").unwrap_or(N_BARRIDO_DEFECTO as f64);
    let tope = rejilla().len();
    if n.fract() == 0.0 && (1.0..=tope as f64).contains(&n) {
        Ok(n as usize)
    } else {
        Err(Error::Invalid(format!(
            "'n' debe ser un entero entre 1 y {tope}"
        )))
    }
}

/// One orientation with its specific yield and monthly energy, for tool output.
fn orientacion_json(o: Orientacion, p: &Produccion) -> Value {
    json!({
        "inclinacion": o.inclinacion,
        "azimut": o.azimut,
        "especifica_kwh_kwp": redondear(p.especifica, 1),
        "anual_kwh": redondear(p.anual, 0),
        "mensual_kwh": p.mensual.iter().map(|m| redondear(*m, 0)).collect::<Vec<f64>>(),
    })
}

/// Assumptions of the local model, echoed in every pre-estimate.
fn supuestos() -> Value {
    json!({
        "modelo": "local: Spencer, Collares-Pereira-Rabl, cielo isotrópico (sin PVsyst ni licencia)",
        "gamma_potencia_por_K": GAMMA_POTENCIA,
        "noct_c": NOCT,
        "perdidas_sistema": PERDIDAS_SISTEMA,
        "albedo": ALBEDO,
    })
}

/// Copies the optional `workspace` argument into the arguments of a sub-call.
fn con_workspace(args: &Value, mut destino: Value) -> Value {
    if let Some(w) = opt_str(args, &["workspace"]) {
        destino["workspace"] = json!(w);
    }
    destino
}

/// Specific yield and monthly energy of one orientation, or the best orientations of the
/// sweep when no `tilt` or `azimut` is given. Runs locally and uses no trial execution.
pub(crate) fn preestimar(_paths: &Paths, args: &Value) -> Result<Value> {
    let kwp = kwp_de(args)?;
    let sitio = sitio_de(args)?;
    let clima = Clima::de_nasa(sitio.latitud, sitio.longitud)?;
    match orientacion_de(args, sitio.latitud)? {
        Some(o) => {
            let p = produccion(&sitio, &clima, o, kwp);
            Ok(json!({"resultado": orientacion_json(o, &p), "supuestos": supuestos()}))
        }
        None => {
            let top: Vec<Value> = ranking(&sitio, &clima, kwp, TOP_PREESTIMACION)
                .iter()
                .map(|(o, p)| orientacion_json(*o, p))
                .collect();
            Ok(json!({"ranking": top, "supuestos": supuestos()}))
        }
    }
}

/// Human-readable confirmation plan: one line per orientation to simulate.
fn plan_de(mejores: &[(Orientacion, Produccion)]) -> Vec<String> {
    mejores
        .iter()
        .map(|(o, p)| {
            format!(
                "tilt {} azim {}: {} kWh/kWp",
                o.inclinacion,
                o.azimut,
                redondear(p.especifica, 1)
            )
        })
        .collect()
}

/// The batch generated for a sweep and the project variant it belongs to.
struct SolicitudLote<'a> {
    args: &'a Value,
    proyecto: &'a str,
    variante: &'a str,
    lote: &'a Value,
}

/// Simulate a generated batch. The budget gate already passed (or was confirmed):
/// do not ask twice.
fn simular_lote(
    paths: &Paths,
    solicitud: &SolicitudLote<'_>,
    seguimiento: &SeguimientoTarea<'_>,
) -> Result<Value> {
    let mut sim_args = json!({
        "proyecto": solicitud.proyecto,
        "variante": solicitud.variante,
        "batch_params_file": solicitud.lote["batch_params_file"],
        "confirmar": true,
    });
    if let Some(t) = solicitud.args.get("timeout_s") {
        sim_args["timeout_s"] = t.clone();
    }
    if let Some(rvt) = solicitud
        .lote
        .get("batch_rvt_file")
        .filter(|v| !v.is_null())
    {
        sim_args["batch_rvt_file"] = rvt.clone();
    }
    lanzar_simulacion_seguida(paths, &con_workspace(solicitud.args, sim_args), seguimiento)
}

/// Sweeps the orientations locally, then simulates the best `n` as one batch run of the
/// project variant. Returns the confirmation request instead while the trial budget is low.
pub(crate) fn barrido(
    paths: &Paths,
    args: &Value,
    seguimiento: &SeguimientoTarea<'_>,
) -> Result<Value> {
    let proyecto = req_str(args, &["proyecto"], "nombre del proyecto")?;
    let variante = opt_str(args, &["variante"]).unwrap_or("VC0");
    let n = numero_de_orientaciones(args)?;
    let kwp = kwp_de(args)?;
    let sitio = sitio_de(args)?;
    let clima = Clima::de_nasa(sitio.latitud, sitio.longitud)?;
    let mejores = ranking(&sitio, &clima, kwp, n);
    let ranking_json: Vec<Value> = mejores
        .iter()
        .map(|(o, p)| orientacion_json(*o, p))
        .collect();
    let plan = plan_de(&mejores);
    let ws = workspace_of(paths, args);
    if let Some(aviso) = ledger::requiere_confirmacion(&ws, args, n as u32, &plan) {
        return Ok(aviso);
    }
    let filas: Vec<Value> = mejores
        .iter()
        .map(|(o, _)| json!({"tilt": o.inclinacion, "azim": o.azimut}))
        .collect();
    let lote = batch::crear_batch(
        paths,
        &con_workspace(
            args,
            json!({"proyecto": proyecto, "variante": variante, "filas": filas}),
        ),
    )?;
    let solicitud = SolicitudLote {
        args,
        proyecto,
        variante,
        lote: &lote,
    };
    let simulacion = simular_lote(paths, &solicitud, seguimiento)?;
    Ok(json!({
        "ranking": ranking_json,
        "lote": lote,
        "simulacion": simulacion,
        "supuestos": supuestos(),
    }))
}

#[cfg(test)]
#[path = "../../tests/unit/simulation/preestimate.rs"]
mod tests;
