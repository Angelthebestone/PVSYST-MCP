//! The one-call design run: preflight checks, the plan, the steps and the undo.

use super::dimensionado::APROXIMACION_DISENO_AUTOMATICO;
use super::parametros::DisenoParametros;
use super::pasos::{Corrida, Fases, OrigenSitioDiseno};
use super::resultado::{desviacion_pct, serializar_resultado};
use super::sitio_del_proyecto::{self, SitioDelDiseno};
use super::{meteonorm, proyecto, simulacion, sitio};
use crate::adapters::ledger;
use crate::aproximaciones;
use crate::config::Paths;
use crate::error::{Error, Result};
use crate::idioma::Texto;
use crate::sites;
use serde_json::{json, Value};

/// One-call end-to-end: coordinates (+ kWp) → site → project → sizing →
/// simulation → KPIs. Creating a new site and simulating are trial executions:
/// when the trial budget would be spent, the call returns the plan instead
/// unless `confirmar` is true.
pub(crate) fn disenar_sistema(paths: &Paths, args: &Value) -> Result<Value> {
    let params = DisenoParametros::parse(args)?;
    exigir_dimensionado_sin_gui(&params)?;
    meteonorm::exigir_si_sintetica(
        meteonorm::bases_de_instalacion(paths).as_deref(),
        params.simulacion.simular && params.simulacion.synthetic_weather,
        paths.idioma,
    )?;
    let paths = &Paths {
        workspace: params
            .workspace
            .clone()
            .unwrap_or_else(|| paths.workspace.clone()),
        ..paths.clone()
    };

    let es_aproximacion = params.dimensionado.via_aproximacion
        && !(params.dimensionado.n_mod_serie.is_some() && params.dimensionado.n_strings.is_some());
    if es_aproximacion {
        aproximaciones::exigir(paths, &APROXIMACION_DISENO_AUTOMATICO)?;
    }

    let busqueda = sites::reuse::Busqueda {
        coordenadas: (params.ubicacion.latitud, params.ubicacion.longitud),
        altitud: params.ubicacion.altitud,
        nombre: None,
    };
    let reusable = sites::reuse::buscar_existente(&paths.workspace, busqueda).is_some();
    let fases = Fases {
        crear_sitio: !reusable,
        simular: params.simulacion.simular,
    };
    if let Some(aviso) = aviso_previo(paths, args, &params, &fases) {
        return Ok(aviso);
    }
    proyecto::ensure_template(paths, &params.proyecto.plantilla)?;

    let origen = if reusable {
        OrigenSitioDiseno::Reutilizado
    } else {
        OrigenSitioDiseno::Nuevo
    };
    let c = correr_transaccional(paths, &params, origen)?;
    let mut res = serializar_resultado(&params, &c);
    if es_aproximacion {
        aproximaciones::marcar(&mut res, &APROXIMACION_DISENO_AUTOMATICO, paths.idioma);
    }
    Ok(res)
}

/// A grid design without both `n_mod_serie` and `n_strings` would be sized by PVsyst's GUI on the
/// real workspace, which failed on every trial: reject it before any work starts. A stand-alone
/// design without both would only fail later, in the sizing step, after the site and the project
/// exist, so it is refused here too instead of leaving half the work behind.
pub(super) fn exigir_dimensionado_sin_gui(params: &DisenoParametros) -> Result<()> {
    let d = &params.dimensionado;
    let completo = d.n_mod_serie.is_some() && d.n_strings.is_some();
    if completo || d.via_aproximacion {
        return Ok(());
    }
    Err(Error::Invalid(
        "Sin 'n_mod_serie' y 'n_strings' el dimensionado no es fiable (red: lo haría la GUI de PVsyst; aislado: no hay pre-dimensionado): pasa ambos o usa via:'aproximacion'".into(),
    ))
}

/// Runs the steps; when one fails, undoes this call on the project's own files.
fn correr_transaccional(
    paths: &Paths,
    params: &DisenoParametros,
    origen: OrigenSitioDiseno,
) -> Result<Corrida> {
    let foto = proyecto::FotoProyecto::tomar(paths, &params.proyecto.nombre)?;
    correr_pasos(paths, params, origen).inspect_err(|_| foto.deshacer(paths))
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
    sitio::exigir_sitio_creado(&paso_sitio)?;
    let paso_proyecto = proyecto::ejecutar_paso_proyecto(paths, params)?;
    sitio_del_proyecto::vincular_sitio(
        paths,
        &SitioDelDiseno {
            nombre: &params.proyecto.nombre,
            variante: &params.variante,
            sit_file: &paso_sitio.sit_file,
        },
    )?;
    let paso_orientacion = proyecto::ejecutar_paso_orientacion(paths, params)?;
    let paso_dimensionado = super::dimensionado::ejecutar_paso_dimensionado(paths, params)?;
    exigir_tolerancia_kwp(
        params.dimensionado.kwp,
        params.dimensionado.tolerancia_kwp_pct,
        paso_dimensionado.kwp_realizado,
    )?;
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

/// The answer to give instead of running: a dry run's plan, or the trial
/// confirmation gate. `None` when the run may proceed.
fn aviso_previo(
    paths: &Paths,
    args: &Value,
    params: &DisenoParametros,
    fases: &Fases,
) -> Option<Value> {
    let ejecuciones = u32::from(fases.crear_sitio) + u32::from(fases.simular);
    let plan = plan_de(&params.proyecto.nombre, &params.variante, fases);
    if crate::args::flag(args, &["dry_run"]) {
        return Some(plan_de_ejecucion(&PlanDeEjecucion {
            paths,
            plan: &plan,
            ejecuciones,
            sitio_reutilizado: !fases.crear_sitio,
        }));
    }
    ledger::requiere_confirmacion(&paths.workspace, args, ejecuciones, &plan)
}

/// What the call would do and what it would cost, without doing any of it.
struct PlanDeEjecucion<'a> {
    paths: &'a Paths,
    plan: &'a [String],
    ejecuciones: u32,
    sitio_reutilizado: bool,
}

fn plan_de_ejecucion(p: &PlanDeEjecucion<'_>) -> Value {
    json!({
        "dry_run": true,
        "plan": p.plan,
        "sitio_reutilizado": p.sitio_reutilizado,
        "costo_estimado": ledger::costo_estimado(&p.paths.workspace, p.ejecuciones),
        "siguiente_paso": Texto {
            es: "repite la misma llamada sin dry_run",
            en: "repeat the same call without dry_run",
        }
        .en_idioma(p.paths.idioma),
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

/// Rejects a realised kWp further from the target than the tolerance allows.
fn exigir_tolerancia_kwp(objetivo: f64, tolerancia_pct: f64, realizado: Option<f64>) -> Result<()> {
    let Some(real) = realizado else {
        return Err(Error::Format(
            "El dimensionado no devolvió 'realised_kwp': no se puede comprobar el kWp objetivo"
                .into(),
        ));
    };
    let pct = desviacion_pct(objetivo, real);
    if pct.abs() <= tolerancia_pct {
        return Ok(());
    }
    Err(Error::Invalid(format!(
        "kWp realizado {real:.1} se desvía {pct:+.1} % del objetivo {objetivo:.1} (tolerancia {tolerancia_pct} %): ajusta n_mod_serie/n_strings o pasa 'tolerancia_kwp_pct'"
    )))
}
