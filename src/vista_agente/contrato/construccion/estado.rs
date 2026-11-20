//! The state and the origin of the view, deduced from what the answer says
//! about itself.

use super::super::tipos::{Estado, Origen};
use super::textos_de;
use super::Contexto;
use serde_json::Value;

/// Tools whose answers describe the workspace or the MCP, not a PVsyst figure.
const TOOLS_DEL_MCP: &[&str] = &["entorno", "workspace", "listar"];
/// Value of `origen` that marks an approximation computed by the MCP.
const ORIGEN_APROXIMACION: &str = "aproximacion_mcp";

pub(super) fn origen_de(ctx: &Contexto<'_>, salida: &Value) -> Origen {
    if salida.get("origen").and_then(Value::as_str) == Some(ORIGEN_APROXIMACION) {
        return Origen::AproximacionMcp;
    }
    if TOOLS_DEL_MCP.contains(&ctx.tool) {
        return Origen::Mcp;
    }
    Origen::Pvsyst
}

fn algun_paso_ok(salida: &Value) -> bool {
    salida
        .get("pasos")
        .and_then(Value::as_array)
        .is_some_and(|pasos| {
            pasos
                .iter()
                .any(|p| p.get("ok").and_then(Value::as_bool) == Some(true))
        })
}

fn es_verdadero(salida: &Value, clave: &str) -> bool {
    salida.get(clave).and_then(Value::as_bool) == Some(true)
}

/// A PVsystCLI run that did not succeed: the handlers answer `Ok` with these keys.
/// A run that produced its output file succeeded even if the process exited with an
/// error: its `detected_error` is then a warning, not a failure.
pub(super) fn falla_la_cli(salida: &Value) -> bool {
    let produjo_salida =
        es_verdadero(salida, "success") || es_verdadero(salida, "confirmed_by_output_file");
    salida.get("success").and_then(Value::as_bool) == Some(false)
        || es_verdadero(salida, "timed_out")
        || (!produjo_salida && !textos_de(salida, "detected_error").is_empty())
}

/// Keys that name a failure inside a successful reply.
fn nombra_fallo(salida: &Value) -> bool {
    !textos_de(salida, "errores").is_empty() || !textos_de(salida, "error").is_empty()
}

/// Keys that name values that did not match what was asked for.
fn nombra_diferencia(salida: &Value) -> bool {
    salida
        .get("campos_que_difieren")
        .and_then(Value::as_u64)
        .is_some_and(|n| n > 0)
        || salida
            .get("diferencias_con_la_ficha")
            .and_then(Value::as_array)
            .is_some_and(|d| !d.is_empty())
}

/// Keys that say the answer is missing part of what was asked for. An incomplete
/// answer must name what failed or differed: `completo:false` alone only warns
/// (a legacy MEF whose binary columns are not decoded), and calling that a
/// failure would invent a reason.
fn es_incompleta(salida: &Value) -> bool {
    nombra_fallo(salida) || nombra_diferencia(salida)
}

/// `ok:false` or a failed CLI run marks a failure inside a successful reply;
/// an `errores` list, `error` text or a named difference mark an incomplete one.
/// It is partial when a step worked or the reply never said it failed.
pub(super) fn estado_de(salida: &Value) -> Estado {
    let fallo = salida.get("ok").and_then(Value::as_bool) == Some(false) || falla_la_cli(salida);
    let con_errores = es_incompleta(salida);
    if !fallo && !con_errores {
        return Estado::Ok;
    }
    if algun_paso_ok(salida) || !fallo {
        return Estado::Parcial;
    }
    Estado::Error
}
