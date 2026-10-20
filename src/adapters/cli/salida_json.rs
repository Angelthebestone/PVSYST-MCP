//! Standard JSON envelope for PVsystCLI executions.

use super::clasificacion::{clasificar_salida, confirmed, is_license_banner, Clasificacion};
use crate::state::CliOutput;
use serde_json::{json, Value};

/// Cantidad de líneas finales retenidas de stdout en el sobre JSON.
const LINEAS_TAIL_STDOUT: usize = 60;

/// Cantidad de líneas finales retenidas de stderr en el sobre JSON.
const LINEAS_TAIL_STDERR: usize = 20;

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

/// Whether the command succeeded, for output post-processing.
#[derive(Clone, Copy, PartialEq, Eq)]
enum EstadoProceso {
    Exito,
    Fallo,
}

fn filtrar_cuerpo_stdout(stdout: &str, estado: EstadoProceso) -> String {
    let exito = matches!(estado, EstadoProceso::Exito);
    stdout
        .lines()
        .filter(|l| !(exito && is_license_banner(l)))
        .filter(|l| !(exito && l.trim_end().ends_with('%')))
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
    let estado = if success {
        EstadoProceso::Exito
    } else {
        EstadoProceso::Fallo
    };
    let cuerpo = filtrar_cuerpo_stdout(&out.stdout, estado);
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
