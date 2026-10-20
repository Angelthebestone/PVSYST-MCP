//! Ejecución de procesos de `PVsystCLI.exe`.
//!
//! Gestiona el ciclo de vida del subproceso, la captura con límite de memoria,
//! la cancelación cooperativa y la publicación de avance.

use super::captura::{crear_sink, es_truncado, texto_sink, Sink};
use super::linea_comando::{display_command_line, requerir_cli, timeout_de, TIMEOUT_DEFECTO_S};
use super::salida_json::output_json;
use crate::args::{flag, req_str};
use crate::config::Paths;
use crate::error::{Error, Result};
use crate::format::value_text;
use crate::progreso::SeguimientoTarea;
pub(crate) use crate::state::CliOutput;
use serde_json::{json, Value};
use std::process::{Child, Command, Stdio};
use std::sync::Arc;
use std::thread;
use std::time::{Duration, Instant};

/// Intervalo de sondeo activo para el estado de finalización del subproceso.
const INTERVALO_ESPERA_PROCESO: Duration = Duration::from_millis(50);

/// Valor inicial de "todavía no se informó ningún porcentaje".
const SIN_PROGRESO: f64 = -1.0;

/// Cantidad de bytes ya escaneados en la ronda anterior.
const SIN_BYTES_ESCANEADOS: usize = 0;

/// Parámetros de ejecución agrupados para cumplir con el límite de parámetros.
pub(crate) struct ParametrosEjecucion<'a> {
    pub(crate) paths: &'a Paths,
    pub(crate) cli_args: &'a [String],
    pub(crate) timeout: Duration,
    pub(crate) seguimiento: &'a SeguimientoTarea<'a>,
}

/// Ejecuta PVsystCLI de forma síncrona esperando hasta alcanzar `timeout`.
pub(crate) fn run(paths: &Paths, cli_args: &[String], timeout: Duration) -> Result<CliOutput> {
    let seguimiento = SeguimientoTarea::inerte();
    run_seguido(&ParametrosEjecucion {
        paths,
        cli_args,
        timeout,
        seguimiento: &seguimiento,
    })
}

/// Ejecuta PVsystCLI reportando avance y atendiendo la cancelación del cliente.
pub(crate) fn run_seguido(params: &ParametrosEjecucion<'_>) -> Result<CliOutput> {
    let comando = params.cli_args.first().map_or("", String::as_str);
    tracing::info!(
        comando,
        timeout_s = params.timeout.as_secs(),
        "launching PVsystCLI"
    );
    let out_sink = crear_sink();
    let out = run_inner(params, &out_sink)?;
    if out.timed_out {
        tracing::warn!(comando, "PVsystCLI timed out and was killed");
    } else {
        tracing::info!(comando, exit_code = ?out.exit_code, "PVsystCLI finished");
    }
    Ok(out)
}

pub(crate) fn run_inner(params: &ParametrosEjecucion<'_>, out_sink: &Sink) -> Result<CliOutput> {
    requerir_cli(params.paths)?;
    let mut child = Command::new(&params.paths.cli_exe)
        .args(params.cli_args)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|e| {
            Error::io(
                format!("Failed to launch {}", params.paths.cli_exe.display()),
                e,
            )
        })?;

    let err_sink = crear_sink();
    let reader_out = super::captura::drain(child.stdout.take(), Arc::clone(out_sink), "stdout");
    let reader_err = super::captura::drain(child.stderr.take(), Arc::clone(&err_sink), "stderr");

    let espera = EsperaProceso {
        timeout: params.timeout,
        sink: out_sink,
        seguimiento: params.seguimiento,
    };
    let resultado = esperar_proceso(&mut child, &espera);
    reader_out
        .join()
        .map_err(|_| Error::Cli("Hilo lector de stdout terminó con pánico".to_string()))??;
    reader_err
        .join()
        .map_err(|_| Error::Cli("Hilo lector de stderr terminó con pánico".to_string()))??;
    let (exit_code, timed_out) = resultado?;

    Ok(CliOutput {
        exit_code,
        stdout: texto_sink(out_sink)?,
        stderr: texto_sink(&err_sink)?,
        command_line: display_command_line(
            &params.paths.cli_exe.to_string_lossy(),
            params.cli_args,
        ),
        timed_out,
        timeout_s: params.timeout.as_secs(),
        stdout_truncado: es_truncado(out_sink)?,
        stderr_truncado: es_truncado(&err_sink)?,
    })
}

/// What one wait loop needs: the deadline, the captured stdout and the tracker.
struct EsperaProceso<'a> {
    timeout: Duration,
    sink: &'a Sink,
    seguimiento: &'a SeguimientoTarea<'a>,
}

fn esperar_proceso(child: &mut Child, espera: &EsperaProceso<'_>) -> Result<(Option<i32>, bool)> {
    let deadline = Instant::now() + espera.timeout;
    let mut ultimo_informado = SIN_PROGRESO;
    let mut bytes_escaneados = SIN_BYTES_ESCANEADOS;
    loop {
        if espera.seguimiento.esta_cancelado() {
            let _ = child.kill();
            let _ = child.wait();
            return Err(Error::Cli("Simulación cancelada por el cliente".into()));
        }
        if let Some(status) = child
            .try_wait()
            .map_err(|e| Error::io("Waiting for PVsystCLI", e))?
        {
            return Ok((status.code(), false));
        }
        if Instant::now() >= deadline {
            let _ = child.kill();
            let _ = child.wait();
            return Ok((None, true));
        }
        // Solo se copia el stdout acumulado cuando el lector añadió bytes nuevos.
        let longitud = super::captura::longitud_sink(espera.sink);
        if longitud != bytes_escaneados {
            bytes_escaneados = longitud;
            let avance = texto_sink(espera.sink)
                .ok()
                .as_deref()
                .and_then(super::trabajo::progreso);
            if let Some(porcentaje) = avance.filter(|p| *p > ultimo_informado) {
                espera
                    .seguimiento
                    .informar_progreso(porcentaje, "Simulando");
                ultimo_informado = porcentaje;
            }
        }
        thread::sleep(INTERVALO_ESPERA_PROCESO);
    }
}

/// Ejecuta cualquier comando con argumentos en crudo (passthrough de `pvsyst_cli`).
pub(crate) fn ejecutar(paths: &Paths, args: &Value) -> Result<Value> {
    let comando = req_str(
        args,
        &["comando", "command"],
        "e.g. \"help\", \"version\", \"lic-info\"",
    )?;

    let mut cli_args: Vec<String> = vec![comando.to_string()];
    if let Some(opts) = args.get("opciones").and_then(Value::as_array) {
        for o in opts {
            if let Some(s) = o.as_str() {
                cli_args.push(s.to_string());
            }
        }
    }
    if let Some(opts) = args.get("opciones").and_then(Value::as_object) {
        for (k, v) in opts {
            let val = value_text(v);
            let flag = k.trim_start_matches('-');
            cli_args.push(format!("--{flag}:{val}"));
        }
    }

    let dry_run = flag(args, &["dry_run"]);
    let command_line = display_command_line(&paths.cli_exe.to_string_lossy(), &cli_args);
    if dry_run {
        return Ok(json!({ "dry_run": true, "command_line": command_line }));
    }
    requerir_cli(paths)?;
    let out = run(paths, &cli_args, timeout_de(args, TIMEOUT_DEFECTO_S))?;
    Ok(output_json(&out, comando, &json!({ "command": comando })))
}

#[cfg(test)]
#[path = "../../../tests/unit/adapters/cli/proceso.rs"]
mod tests;
