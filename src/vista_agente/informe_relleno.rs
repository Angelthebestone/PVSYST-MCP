//! What `rellenar` reports for each field and for the whole fill: a field typed and
//! read back, one that reads differently, one that failed, and the summary keys the
//! answer contract turns into `parcial`.

use super::dominio::CampoEsquema;
use super::plan::coincide;
use crate::error::Error;
use serde_json::{json, Value};

const ESTADO_OK: &str = "ok";
const ESTADO_DISTINTO: &str = "distinto";
const ESTADO_SIMULADO: &str = "simulado";
const ESTADO_ERROR: &str = "error";

/// One field: `simulado` without a read-back, else `ok` or `distinto` against the request.
pub(super) fn paso_json(
    campo: &CampoEsquema,
    (pedido, antes): (&str, &str),
    despues: Option<&str>,
) -> Value {
    let estado = match despues {
        None => ESTADO_SIMULADO,
        Some(d) if coincide(campo, pedido, d) => ESTADO_OK,
        Some(_) => ESTADO_DISTINTO,
    };
    json!({
        "clave": campo.clave,
        "unidad": campo.unidad,
        "pedido": pedido,
        "antes": antes,
        "despues": despues,
        "estado": estado,
    })
}

/// A field whose typing failed; the fill stops after it.
pub(super) fn paso_fallido(clave: &str, error: &Error) -> Value {
    json!({ "clave": clave, "estado": ESTADO_ERROR, "error": error.to_string() })
}

/// A tab that could not be shown or read after earlier tabs were already typed.
pub(super) fn pagina_fallida(pagina: usize, error: &Error) -> Value {
    json!({ "pagina": pagina, "estado": ESTADO_ERROR, "error": error.to_string() })
}

pub(super) fn es_fallido(resultado: &Value) -> bool {
    resultado["estado"] == ESTADO_ERROR
}

/// Adds `campos_que_difieren`, and `completo:false` with `errores` when a field failed,
/// so a fill that only partly took is not answered as `ok`. O(fields).
pub(super) fn resumir_relleno(salida: &mut Value) {
    let resultados = salida["resultados"].as_array().cloned().unwrap_or_default();
    let distintos = resultados
        .iter()
        .filter(|r| r["estado"] == ESTADO_DISTINTO)
        .count();
    let errores: Vec<Value> = resultados
        .iter()
        .filter(|r| es_fallido(r))
        .map(|r| r["error"].clone())
        .collect();
    salida["campos_que_difieren"] = json!(distintos);
    if !errores.is_empty() {
        salida["completo"] = json!(false);
        salida["errores"] = Value::Array(errores);
    }
}

#[cfg(test)]
#[path = "../../tests/unit/vista_agente/informe_relleno.rs"]
mod tests;
