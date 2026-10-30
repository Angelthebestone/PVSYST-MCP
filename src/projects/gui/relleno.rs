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
        .find(|r| r["estado"] != "ok");
    if let Some(r) = distinto {
        return Err(Error::Gui(format!(
            "PVsyst no tomó '{}': pedido {}, leído {}",
            r["clave"], r["pedido"], r["despues"]
        )));
    }
    Ok(respuesta["formulario"].clone())
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
