//! Gestión de trabajos en segundo plano (`simular {segundo_plano:true}`).
//!
//! Permite lanzar ejecuciones asíncronas de `PVsystCLI.exe`, registrar su avance
//! mediante análisis de porcentaje en salida y consultar su estado final.

use super::captura::{crear_sink, texto_sink};
use super::proceso::{run_inner, CliOutput, ParametrosEjecucion};
use crate::config::Paths;
use crate::error::Result;
pub(crate) use crate::state::Trabajo;
use std::sync::Arc;
use std::thread;
use std::time::Duration;

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
    if let Ok(mut mapa) = paths.estado.trabajos.lock() {
        mapa.insert(
            id.clone(),
            Trabajo {
                sink: Arc::clone(&sink),
                resultado: None,
            },
        );
    }
    let paths_clon = paths.clone();
    let args_clon = cli_args.to_vec();
    let id_hilo = id.clone();
    thread::spawn(move || {
        let params = ParametrosEjecucion {
            paths: &paths_clon,
            cli_args: &args_clon,
            timeout,
        };
        let resultado = run_inner(&params, &sink);
        if let Ok(mut mapa) = paths_clon.estado.trabajos.lock() {
            if let Some(t) = mapa.get_mut(&id_hilo) {
                t.resultado = Some(resultado);
            }
        }
    });
    Ok(id)
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
mod tests {
    use super::*;

    #[test]
    fn should_parse_progress_percentage_with_newlines() {
        assert_eq!(
            progreso("Simulation 07/02/90 - 10%\nSimulation 07/02/90 - 35%"),
            Some(35.0)
        );
        assert_eq!(progreso("sin progreso"), None);
    }

    #[test]
    fn should_parse_progress_percentage_with_carriage_returns() {
        assert_eq!(
            progreso("Simulation 07/02/90 - 10%\rSimulation 07/02/90 - 80%\r"),
            Some(80.0)
        );
        assert_eq!(progreso("sin progreso\r"), None);
    }
}
