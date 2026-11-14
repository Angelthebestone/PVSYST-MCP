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
    let changes = edit::changes_from_json(mapa)?;
    edit::reject_invalid_keys(&changes)?;
    Ok(changes)
}

/// A key the root object does not hold would be inserted at the root, outside the
/// object PVsyst reads it from (a module's dimensions live in `pvCommercial`).
fn exigir_campos_existentes(
    raiz: &object::PvNode,
    changes: &[(String, Option<String>)],
) -> Result<()> {
    let nuevas: Vec<&str> = changes
        .iter()
        .filter(|(clave, valor)| valor.is_some() && raiz.field(clave).is_none())
        .map(|(clave, _)| clave.as_str())
        .collect();
    if nuevas.is_empty() {
        return Ok(());
    }
    Err(Error::Invalid(format!(
        "Campos que el componente no tiene: {nuevas:?}. componente editar cambia valores existentes; \
para añadir un campo usa editar accion:campos con el objeto que lo contiene (p. ej. objeto:\"pvCommercial\")"
    )))
}

/// Apply the requested changes to the root object of a component, in place.
pub(super) fn aplicar_cambios(bytes: &mut Vec<u8>, args: &Value) -> Result<Vec<Value>> {
    let changes = cambios_pedidos(args)?;
    if changes.is_empty() {
        return Ok(Vec::new());
    }
    let texto = String::from_utf8_lossy(bytes);
    if object::is_pvobject(&texto) {
        object::validar_estructura(bytes)?;
        let raiz = object::parse_first(&texto)
            .ok_or_else(|| Error::Format("El componente no contiene un PVObject legible".into()))?;
        exigir_campos_existentes(&raiz, &changes)?;
        let (nuevo, outcomes) = edit::edit_fields(bytes, &raiz.obj_type, &changes)?;
        *bytes = nuevo;
        return Ok(outcomes.iter().map(edit::ChangeOutcome::to_json).collect());
    }
    if pan::es_pan_legacy(bytes) {
        let (nuevo, outcomes) = pan::parchear_campos_legacy(bytes, &changes)?;
        *bytes = nuevo;
        return Ok(outcomes.iter().map(edit::ChangeOutcome::to_json).collect());
    }
    Err(Error::Invalid(
        "Target component is binary; field editing is not supported.".into(),
    ))
}
