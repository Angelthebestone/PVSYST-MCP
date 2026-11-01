//! Field changes applied to a component's bytes: text PVObjects through the generic
//! editor, legacy binary `.PAN` modules through their proven Real48 slots.

use super::pan;
use crate::error::{Error, Result};
use crate::format::{edit, object};
use serde_json::Value;

/// Field changes of a component call: `cambios`, or its synonym `campos`.
fn cambios_pedidos(args: &Value) -> Result<Vec<(String, Option<String>)>> {
    let Some(mapa) = args
        .get("cambios")
        .or_else(|| args.get("campos"))
        .and_then(Value::as_object)
    else {
        return Ok(Vec::new());
    };
    let changes = edit::changes_from_json(mapa);
    edit::reject_invalid_keys(&changes)?;
    Ok(changes)
}

/// Apply the requested changes to the root object of a component, in place.
pub(super) fn aplicar_cambios(bytes: &mut Vec<u8>, args: &Value) -> Result<Vec<Value>> {
    let changes = cambios_pedidos(args)?;
    if changes.is_empty() {
        return Ok(Vec::new());
    }
    let raiz = {
        let text = String::from_utf8_lossy(bytes);
        object::is_pvobject(&text).then(|| object::parse_first(&text).map(|r| r.obj_type))
    };
    let (new_bytes, outcomes) = match raiz {
        Some(Some(tipo)) => edit::edit_fields(bytes, &tipo, &changes)?,
        Some(None) => {
            return Err(Error::Format(
                "El componente no contiene un PVObject legible".into(),
            ))
        }
        None if pan::es_pan_legacy(bytes) => pan::parchear_campos_legacy(bytes, &changes)?,
        None => {
            return Err(Error::Invalid(
                "Target component is binary; field editing is not supported.".into(),
            ))
        }
    };
    *bytes = new_bytes;
    Ok(outcomes.iter().map(edit::ChangeOutcome::to_json).collect())
}

#[cfg(test)]
#[path = "../../tests/unit/components/cambios.rs"]
mod tests;
