//! Fills a PVsyst form of the project steps by the keys of its schema (`gui_observar
//! que:esquema`), not by coordinates, and refuses a field PVsyst did not take.

use crate::adapters::gui::conduccion::Formulario;
use crate::config::Paths;
use crate::error::{Error, Result};
use serde_json::{json, Value};

/// Types `valores` ({key: value}) into `formulario`, an open window of entity `entidad`.
/// Returns what PVsyst shows afterwards: the fields of the entity by key.
pub(super) fn rellenar(
    paths: &Paths,
    formulario: &Formulario,
    (entidad, valores): (&str, Value),
) -> Result<Value> {
    let pedido = json!({
        "entidad": entidad,
        "ventana": format!("0x{:X}", formulario.h),
        "valores": valores,
    });
    let respuesta = crate::vista_agente::rellenar(paths, &pedido)?;
    let distinto = respuesta["resultados"]
        .as_array()
        .into_iter()
        .flatten()
        .find(|r| !tomado(r));
    if let Some(r) = distinto {
        return Err(Error::Gui(format!(
            "PVsyst no tomó '{}': pedido {}, leído {}",
            r["clave"], r["pedido"], r["despues"]
        )));
    }
    Ok(respuesta["formulario"].clone())
}

/// Largest relative gap between a number asked for and the one PVsyst shows back: PVsyst keeps
/// the decimals of its own field (12.345 reads back 12.3), which is still the value it took.
const REDONDEO_DE_PVSYST: f64 = 0.01;

/// Whether PVsyst took the value of one result of the fill.
fn tomado(resultado: &Value) -> bool {
    if resultado["estado"] == "ok" {
        return true;
    }
    let numero = |v: &Value| {
        v.as_str()
            .and_then(|t| t.trim().replace(',', ".").parse::<f64>().ok())
    };
    match (numero(&resultado["pedido"]), numero(&resultado["despues"])) {
        (Some(pedido), Some(leido)) => (pedido - leido).abs() <= pedido.abs() * REDONDEO_DE_PVSYST,
        _ => false,
    }
}

/// The value of field `clave` in what [`rellenar`] returned.
pub(super) fn valor(formulario: &Value, clave: &str) -> Value {
    formulario["campos"]
        .as_array()
        .into_iter()
        .flatten()
        .find(|c| c["clave"] == clave)
        .map_or(Value::Null, |c| c["valor"].clone())
}

#[cfg(test)]
#[path = "../../../tests/unit/projects/gui/relleno.rs"]
mod tests;
