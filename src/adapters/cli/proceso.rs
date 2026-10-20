//! Ejecución de procesos de `PVsystCLI.exe` y formateo de resultados.
//!
//! Gestiona el ciclo de vida del subproceso, la captura con límite de memoria
//! y la construcción del sobre JSON de respuesta estándar.

use super::captura::{crear_sink, es_truncado, texto_sink, Sink};
use super::clasificacion::{clasificar_salida, confirmed, is_license_banner, Clasificacion};
use super::linea_comando::{display_command_line, requerir_cli, timeout_de, TIMEOUT_DEFECTO_S};
use crate::args::{flag, req_str};
use crate::config::Paths;
use crate::error::{Error, Result};
use crate::format::value_text;
pub(crate) use crate::state::CliOutput;
use serde_json::{json, Value};
use std::process::{Child, Command, Stdio};
use std::sync::Arc;
use std::thread;
use std::time::{Duration, Instant};

/// Intervalo de sondeo activo para el estado de finalización del subproceso.
const INTERVALO_ESPERA_PROCESO: Duration = Duration::from_millis(50);

/// Cantidad de líneas finales retenidas de stdout en el sobre JSON.
const LINEAS_TAIL_STDOUT: usize = 60;

/// Cantidad de líneas finales retenidas de stderr en el sobre JSON.
const LINEAS_TAIL_STDERR: usize = 20;

/// Parámetros de ejecución agrupados para cumplir con el límite de parámetros.
pub(crate) struct ParametrosEjecucion<'a> {
    pub(crate) paths: &'a Paths,
    pub(crate) cli_args: &'a [String],
    pub(crate) timeout: Duration,
}

/// Ejecuta PVsystCLI de forma síncrona esperando hasta alcanzar `timeout`.
pub(crate) fn run(paths: &Paths, cli_args: &[String], timeout: Duration) -> Result<CliOutput> {
    let comando = cli_args.first().map_or("", String::as_str);
    tracing::info!(
        comando,
        timeout_s = timeout.as_secs(),
        "launching PVsystCLI"
    );
    let out_sink = crear_sink();
    let params = ParametrosEjecucion {
        paths,
        cli_args,
        timeout,
    };
    let out = run_inner(&params, &out_sink)?;
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

    let (exit_code, timed_out) = esperar_proceso(&mut child, params.timeout)?;

    reader_out
        .join()
        .map_err(|_| Error::Cli("Hilo lector de stdout terminó con pánico".to_string()))??;
    reader_err
        .join()
        .map_err(|_| Error::Cli("Hilo lector de stderr terminó con pánico".to_string()))??;

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

fn esperar_proceso(child: &mut Child, timeout: Duration) -> Result<(Option<i32>, bool)> {
    let deadline = Instant::now() + timeout;
    loop {
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
        thread::sleep(INTERVALO_ESPERA_PROCESO);
    }
}

pub(crate) fn tail(s: &str, n: usize) -> String {
    let lines: Vec<&str> = s.lines().collect();
    let start = lines.len().saturating_sub(n);
    lines[start..].join("\n")
}

fn determinar_estado_ejecucion(
    out: &CliOutput,
    command: &str,
) -> (&'static str, bool, Option<String>, Option<Clasificacion>) {
    if out.timed_out {
        return (
            "timeout",
            false,
            Some(format!("timeout tras {} s", out.timeout_s)),
            None,
        );
    }
    let clasificacion = clasificar_salida(command, &out.stdout, out.exit_code);
    let (success, err) = match &clasificacion {
        Clasificacion::Exito => (true, None),
        Clasificacion::Error(e) => (false, Some(e.clone())),
        Clasificacion::Ambiguo(m) => (false, Some(format!("ambiguo: {m}"))),
    };
    (clasificacion.etiqueta(), success, err, Some(clasificacion))
}

fn filtrar_cuerpo_stdout(stdout: &str, success: bool) -> String {
    stdout
        .lines()
        .filter(|l| !(success && is_license_banner(l)))
        .filter(|l| !(success && l.trim_end().ends_with('%')))
        .collect::<Vec<_>>()
        .join("\n")
}

fn enriquecer_salida_json(
    base: &mut Value,
    out: &CliOutput,
    detected_error: Option<&str>,
    extra: &Value,
) {
    if let Some(trial) = out.stdout.lines().find(|l| l.contains("TRIAL")) {
        base["license"] = json!(trial.trim());
    }
    if let Some(e) = detected_error {
        base["detected_error"] = json!(e);
    }
    if !out.stderr.trim().is_empty() {
        base["stderr_tail"] = json!(tail(&out.stderr, LINEAS_TAIL_STDERR));
    }
    if let (Some(obj), Some(extra_map)) = (base.as_object_mut(), extra.as_object()) {
        for (k, v) in extra_map {
            obj.insert(k.clone(), v.clone());
        }
    }
}

/// Construye el sobre JSON estándar para una ejecución CLI.
pub(crate) fn output_json(out: &CliOutput, command: &str, extra: &Value) -> Value {
    let (etiqueta, success, detected_error, clasificacion) =
        determinar_estado_ejecucion(out, command);
    let cuerpo = filtrar_cuerpo_stdout(&out.stdout, success);
    let mut base = json!({
        "success": success,
        "clasificacion": etiqueta,
        "confirmed_by_output": !out.timed_out && confirmed(command, &out.stdout),
        "exit_code": out.exit_code,
        "command_line": out.command_line,
        "stdout_tail": tail(cuerpo.trim(), LINEAS_TAIL_STDOUT),
        "timed_out": out.timed_out,
        "stdout_truncado": out.stdout_truncado,
        "stderr_truncado": out.stderr_truncado,
    });
    if let Some(Clasificacion::Ambiguo(motivo)) = &clasificacion {
        base["ambiguo"] = json!(true);
        base["motivo_ambiguedad"] = json!(motivo);
    }
    enriquecer_salida_json(&mut base, out, detected_error.as_deref(), extra);
    base
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
