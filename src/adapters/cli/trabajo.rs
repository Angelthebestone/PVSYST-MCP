//! Gestión de trabajos en segundo plano (`simular {segundo_plano:true}`).
//!
//! Permite lanzar ejecuciones asíncronas de `PVsystCLI.exe`, registrar su avance
//! mediante análisis de porcentaje en salida y consultar su estado final.

use super::captura::{crear_sink, texto_sink};
use super::proceso::{run_inner, CliOutput, ParametrosEjecucion};
use crate::config::Paths;
use crate::error::{Error, Result};
use crate::idioma::{Idioma, Texto};
use crate::progreso::SeguimientoTarea;
pub(crate) use crate::state::Trabajo;
use std::collections::HashMap;
use std::sync::Arc;
use std::thread;
use std::time::Duration;

const MUTEX_ENVENENADO: Texto = Texto {
    es: "Mutex de trabajos envenenado: no se lanzó el trabajo",
    en: "Job mutex poisoned: the job was not started",
};
const HILO_CON_PANICO: Texto = Texto {
    es: "El hilo del trabajo terminó con pánico antes de registrar su resultado",
    en: "The job thread panicked before recording its result",
};

/// Cuántos trabajos terminados se conservan: cada uno retiene hasta 2 MiB de salida
/// y nadie los borraba, así que una sesión larga crecía sin límite.
const TRABAJOS_TERMINADOS_CONSERVADOS: usize = 8;

/// Estado observable de un trabajo en segundo plano.
pub(crate) struct EstadoTrabajo {
    pub terminado: bool,
    pub estado: String,
    pub progreso_pct: Option<f64>,
    pub salida: Option<CliOutput>,
    pub error: Option<String>,
}

/// Inicia la ejecución de PVsystCLI en un hilo dedicado y devuelve el identificador de trabajo.
pub(crate) fn run_en_segundo_plano(
    paths: &Paths,
    cli_args: &[String],
    timeout: Duration,
) -> Result<String> {
    let id = paths.estado.siguiente_trabajo();
    let sink = crear_sink();
    let mut trabajos = paths
        .estado
        .trabajos
        .lock()
        .map_err(|_| Error::Cli(MUTEX_ENVENENADO.en_idioma(paths.idioma).to_string()))?;
    trabajos.insert(
        id.clone(),
        Trabajo {
            sink: Arc::clone(&sink),
            resultado: None,
        },
    );
    liberar_trabajos_terminados(&mut trabajos);
    drop(trabajos);
    let paths_clon = paths.clone();
    let args_clon = cli_args.to_vec();
    let id_hilo = id.clone();
    thread::spawn(move || {
        let seguimiento = SeguimientoTarea::inerte();
        let params = ParametrosEjecucion {
            paths: &paths_clon,
            cli_args: &args_clon,
            timeout,
            seguimiento: &seguimiento,
        };
        let resultado = sin_panico(|| run_inner(&params, &sink));
        if let Ok(mut mapa) = paths_clon.estado.trabajos.lock() {
            if let Some(t) = mapa.get_mut(&id_hilo) {
                t.resultado = Some(resultado);
            }
        }
    });
    Ok(id)
}

/// Borra los trabajos terminados más antiguos, conservando los últimos: sus
/// resultados ya se leyeron y su salida solo ocupa memoria.
fn liberar_trabajos_terminados(trabajos: &mut HashMap<String, Trabajo>) {
    let mut terminados: Vec<(u64, String)> = trabajos
        .iter()
        .filter(|(_, t)| t.resultado.is_some())
        .filter_map(|(id, _)| Some((id.strip_prefix("sim-")?.parse().ok()?, id.clone())))
        .collect();
    let sobran = terminados
        .len()
        .saturating_sub(TRABAJOS_TERMINADOS_CONSERVADOS);
    if sobran == 0 {
        return;
    }
    terminados.sort();
    for (_, id) in terminados.into_iter().take(sobran) {
        trabajos.remove(&id);
    }
}

/// Runs a job body so that a panic becomes an error result: otherwise the job would
/// never record an outcome and `estado_trabajo` would answer "ejecutando" forever.
pub(crate) fn sin_panico(trabajo: impl FnOnce() -> Result<CliOutput>) -> Result<CliOutput> {
    std::panic::catch_unwind(std::panic::AssertUnwindSafe(trabajo)).unwrap_or_else(|_| {
        Err(Error::Cli(
            HILO_CON_PANICO.en_idioma(Idioma::de_entorno()).to_string(),
        ))
    })
}

/// Consulta el estado actual del trabajo `id`, o `None` si no existe en memoria.
pub(crate) fn estado_trabajo(paths: &Paths, id: &str) -> Option<EstadoTrabajo> {
    let mapa = paths.estado.trabajos.lock().ok()?;
    let t = mapa.get(id)?;
    let progreso_pct = texto_sink(&t.sink).ok().as_deref().and_then(progreso);
    Some(match &t.resultado {
        None => EstadoTrabajo {
            terminado: false,
            estado: "ejecutando".to_string(),
            progreso_pct,
            salida: None,
            error: None,
        },
        Some(Ok(out)) => EstadoTrabajo {
            terminado: true,
            estado: if out.timed_out {
                "timeout"
            } else {
                "terminado"
            }
            .to_string(),
            progreso_pct,
            salida: Some(out.clone()),
            error: None,
        },
        Some(Err(e)) => EstadoTrabajo {
            terminado: true,
            estado: "error".to_string(),
            progreso_pct,
            salida: None,
            error: Some(e.to_string()),
        },
    })
}

/// Extrae el último porcentaje de avance en el flujo de salida ('NN%').
pub(crate) fn progreso(salida: &str) -> Option<f64> {
    salida.split(['\n', '\r']).rev().find_map(|l| {
        l.trim_end()
            .strip_suffix('%')?
            .rsplit(char::is_whitespace)
            .next()?
            .parse::<f64>()
            .ok()
    })
}

#[cfg(test)]
#[path = "../../../tests/unit/adapters/cli/trabajo.rs"]
mod tests;
