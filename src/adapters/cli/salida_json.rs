//! Standard JSON envelope for PVsystCLI executions.

use super::clasificacion::{
    clasificar_salida, confirmed, es_comando_informativo, is_license_banner, Clasificacion,
};
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
    // A command whose worker thread died confirms nothing: the Admin log says why,
    // and an agent that reads «ambiguo» retries and spends another trial run.
    let clasificacion = match (clasificacion, &out.crash) {
        (Clasificacion::Ambiguo(_), Some(motivo)) => {
            Clasificacion::Error(format!("crash: {motivo}"))
        }
        (otra, _) => otra,
    };
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

/// Stdout without the per-run noise. The license banner is only noise for the
/// commands that act (a `help` answer *is* the list of `lic-activate`), and the
/// progress percentages only exist in a simulation.
fn filtrar_cuerpo_stdout(stdout: &str, command: &str, estado: EstadoProceso) -> String {
    let exito = matches!(estado, EstadoProceso::Exito);
    let quitar_licencia = exito && !es_comando_informativo(command);
    let quitar_progreso = exito && command.eq_ignore_ascii_case("run-simulation");
    stdout
        .lines()
        .filter(|l| !(quitar_licencia && is_license_banner(l)))
        .filter(|l| !(quitar_progreso && l.trim_end().ends_with('%')))
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
    if let Some(crash) = &out.crash {
        base["crash"] = json!(crash);
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
    let cuerpo = filtrar_cuerpo_stdout(&out.stdout, command, estado);
    let cuerpo_recortado = cuerpo.trim();
    let mut base = json!({
        "success": success,
        "clasificacion": etiqueta,
        "confirmed_by_output": !out.timed_out && confirmed(command, &out.stdout),
        "exit_code": out.exit_code,
        "command_line": out.command_line,
        "stdout_tail": tail(cuerpo_recortado, LINEAS_TAIL_STDOUT),
        "stdout_tail_truncado": cuerpo_recortado.lines().count() > LINEAS_TAIL_STDOUT,
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
