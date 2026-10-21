//! High-level "easy mode": environment preflight, a one-call end-to-end design
//! workflow, and headline KPI extraction.
//!
//! The point is that an LLM (or a user who does not know PVsyst) can accomplish
//! a full study with **one** tool call instead of orchestrating a dozen, which
//! also removes most of the per-turn latency.

mod baterias;
mod diagnostico;
mod dimensionado;
mod lectura_argumentos;
mod parametros;
mod pasos;
mod proyecto;
mod serie_plantilla;
mod simulacion;
mod sitio;

#[cfg(test)]
#[path = "../../tests/unit/easy_mode/tests.rs"]
mod tests;

pub(crate) use diagnostico::preflight;
pub(crate) use simulacion::leer_kpis;

use parametros::DisenoParametros;

use crate::adapters::ledger;
use crate::config::Paths;
use crate::error::Result;
use crate::sites;
use pasos::{Corrida, Fases, OrigenSitioDiseno};
use serde_json::{json, Value};

const MAX_DEVIATION_PCT: f64 = 5.0;

/// One-call end-to-end: coordinates (+ kWp) → site → project → sizing →
/// simulation → KPIs. Creating a new site and simulating are trial executions:
/// when the trial budget would be spent, the call returns the plan instead
/// unless `confirmar` is true.
pub(crate) fn disenar_sistema(paths: &Paths, args: &Value) -> Result<Value> {
    let params = DisenoParametros::parse(args)?;
    let paths = &Paths {
        workspace: params
            .workspace
            .clone()
            .unwrap_or_else(|| paths.workspace.clone()),
        ..paths.clone()
    };

    let altitud = params.ubicacion.altitud;
    let reusable = sites::reuse::buscar_existente(
        &paths.workspace,
        params.ubicacion.latitud,
        params.ubicacion.longitud,
        altitud,
    )
    .is_some();
    let fases = Fases {
        crear_sitio: !reusable,
        simular: params.simulacion.simular,
    };
    let ejecuciones = u32::from(fases.crear_sitio) + u32::from(fases.simular);
    let plan = plan_de(&params.proyecto.nombre, &params.variante, &fases);
    if let Some(aviso) = ledger::requiere_confirmacion(&paths.workspace, args, ejecuciones, &plan) {
        return Ok(aviso);
    }
    proyecto::ensure_template(paths, &params.proyecto.plantilla)?;

    let origen = if reusable {
        OrigenSitioDiseno::Reutilizado
    } else {
        OrigenSitioDiseno::Nuevo
    };
    let c = correr_pasos(paths, &params, origen)?;
    Ok(serializar_resultado(&params, &c))
}

fn serializar_resultado(params: &DisenoParametros, c: &Corrida) -> Value {
    let pasos_json = c.a_pasos_json();
    let kpis_error_str = c.simulacion.as_ref().and_then(|s| s.kpis_error.as_deref());
    let errores = errores_de(&pasos_json, kpis_error_str);
    let mut avisos = avisos_de(&pasos_json);
    let kwp_realizado_val = c
        .dimensionado
        .kwp_realizado
        .map_or(Value::Null, |k| json!(k));
    if let Some(a) = aviso_desviacion(params.dimensionado.kwp, &kwp_realizado_val) {
        avisos.push(a);
    }

    let site_info = json!({
        "name": c.sitio.site_name,
        "file": c.sitio.sit_file,
        "weather_source": c.sitio.weather_source,
        "annual_ghi_kwh_m2": c.sitio.annual_ghi_kwh_m2,
        "reutilizado": c.sitio.reutilizado,
    });
    let sim_info = c
        .simulacion
        .as_ref()
        .map_or(Value::Null, |s| s.sim_info.clone());
    let kpis_val = c
        .simulacion
        .as_ref()
        .and_then(|s| s.kpis.as_ref())
        .map_or(Value::Null, |k| json!(k));

    json!({
        "ok": errores.is_empty(),
        "errores": errores,
        "avisos": avisos,
        "proyecto": params.proyecto.nombre,
        "tipo_sistema": params.tipo_sistema.nombre(),
        "variante": params.variante,
        "ubicacion": { "latitud": params.ubicacion.latitud, "longitud": params.ubicacion.longitud },
        "kwp_objetivo": params.dimensionado.kwp,
        "kwp_realizado": kwp_realizado_val,
        "site": site_info,
        "orientacion": { "tilt_deg": c.orientacion.tilt, "azimut_deg": c.orientacion.azimut },
        "proyecto_creado": c.proyecto.raw_response.get("project"),
        "simulacion": sim_info,
        "kpis": kpis_val,
        "pasos": pasos_json,
    })
}

/// Runs site → project → orientation → sizing → (simulation + KPIs) in order.
fn correr_pasos(
    paths: &Paths,
    params: &DisenoParametros,
    sitio_origen: OrigenSitioDiseno,
) -> Result<Corrida> {
    let paso_sitio = sitio::ejecutar_paso_sitio(
        paths,
        &params.ubicacion,
        &params.proyecto.nombre,
        sitio_origen,
    )?;
    let paso_proyecto = proyecto::ejecutar_paso_proyecto(paths, params)?;
    let paso_orientacion = proyecto::ejecutar_paso_orientacion(paths, params)?;
    let paso_dimensionado = dimensionado::ejecutar_paso_dimensionado(paths, params)?;
    let paso_simulacion = if params.simulacion.simular {
        let sim = simulacion::ejecutar_paso_simulacion(paths, params, &paso_sitio.sit_file)?;
        Some(sim)
    } else {
        None
    };

    Ok(Corrida {
        sitio: paso_sitio,
        proyecto: paso_proyecto,
        orientacion: paso_orientacion,
        dimensionado: paso_dimensionado,
        simulacion: paso_simulacion,
    })
}

/// Trial executions the plan will use, for the confirmation gate.
fn plan_de(nombre: &str, variante: &str, fases: &Fases) -> Vec<String> {
    let mut plan = Vec::new();
    if fases.crear_sitio {
        plan.push(format!("crear sitio {nombre}_Site (PVsystCLI create-site)"));
    }
    if fases.simular {
        plan.push(format!(
            "simular {nombre} {variante} (PVsystCLI run-simulation)"
        ));
    }
    plan
}

/// Failed steps and a failed KPI read, as human-readable messages.
pub(crate) fn errores_de(pasos: &[Value], kpis_error: Option<&str>) -> Vec<String> {
    let mut errores: Vec<String> = pasos
        .iter()
        .filter(|p| p.get("ok").and_then(Value::as_bool) != Some(true))
        .map(|p| {
            let paso = p.get("paso").and_then(Value::as_str).unwrap_or("?");
            match p.get("detail").and_then(Value::as_str) {
                Some(d) => format!("paso '{paso}' fallido: {d}"),
                None => format!("paso '{paso}' fallido"),
            }
        })
        .collect();
    if let Some(e) = kpis_error {
        errores.push(format!("leer_kpis: {e}"));
    }
    errores
}

/// Warnings raised by the steps (`pasos[].avisos`), flattened.
pub(crate) fn avisos_de(pasos: &[Value]) -> Vec<String> {
    pasos
        .iter()
        .filter_map(|p| p.get("avisos")?.as_array())
        .flatten()
        .filter_map(Value::as_str)
        .map(str::to_string)
        .collect()
}

/// Warning when the realised kWp is more than 5 % away from the target.
pub(crate) fn aviso_desviacion(objetivo: f64, realizado: &Value) -> Option<String> {
    let real = realizado.as_f64()?;
    let pct = (real - objetivo) / objetivo * 100.0;
    (pct.abs() > MAX_DEVIATION_PCT).then(|| {
        format!("kWp realizado {real:.1} se desvía {pct:+.1} % del objetivo {objetivo:.1}")
    })
}
