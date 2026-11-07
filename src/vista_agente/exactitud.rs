//! A fill that must leave the form exactly as asked: a value PVsyst limited, rejected or did
//! not take is an error, so nothing is saved or sized on top of it.

use super::relleno::rellenar;
use crate::config::Paths;
use crate::error::{Error, Result};
use serde_json::Value;

/// One line per result of a fill that does not read back as asked. Pure.
pub(super) fn diferencias(respuesta: &Value) -> Vec<String> {
    respuesta["resultados"]
        .as_array()
        .into_iter()
        .flatten()
        .filter(|r| r["estado"] != "ok")
        .map(|r| {
            format!(
                "'{}': pedido {}, leído {}",
                r["clave"].as_str().unwrap_or("?"),
                r["pedido"],
                r["despues"]
            )
        })
        .collect()
}

/// [`rellenar`], failing with every field PVsyst did not take when any is left different.
pub(crate) fn rellenar_exacto(paths: &Paths, pedido: &Value) -> Result<Value> {
    let respuesta = rellenar(paths, pedido)?;
    let distintos = diferencias(&respuesta);
    if distintos.is_empty() {
        return Ok(respuesta);
    }
    Err(Error::Gui(format!(
        "PVsyst no tomó {} valor(es): {}",
        distintos.len(),
        distintos.join("; ")
    )))
}

#[cfg(test)]
#[path = "../../tests/unit/vista_agente/exactitud.rs"]
mod tests;
