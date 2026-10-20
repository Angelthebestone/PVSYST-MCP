//! Variant object and field editing.

use crate::args::missing;
use crate::config::Paths;
use crate::error::Result;
use crate::format::edit::{self, EdicionObjeto};
use crate::projects::lifecycle::{abrir_variante, guardar_variante};
use serde_json::{json, Value};

/// Apply field edits to a variant's objects.
/// `ediciones`: [{ "objeto": "pvSubArray", "campos": { "NStringCh": 48 } }]
pub(crate) fn editar_variante(paths: &Paths, args: &Value) -> Result<Value> {
    let ediciones = args
        .get("ediciones")
        .and_then(Value::as_array)
        .ok_or_else(|| missing("ediciones", "array of {objeto, campos}"))?;

    let plan: Vec<EdicionObjeto> = ediciones.iter().map(edicion_de).collect::<Result<_>>()?;
    let abierta = abrir_variante(paths, args)?;
    let (bytes, outcomes) = edit::editar_objetos(&abierta.bytes, &plan)?;
    let report = reporte(&plan, &outcomes);
    let historial_id = guardar_variante(&abierta, &bytes, "editar_variante")?;
    Ok(json!({
        "variant_file": abierta.vpath.to_string_lossy(),
        "historial_id": historial_id,
        "edits": report,
    }))
}

fn edicion_de(ed: &Value) -> Result<EdicionObjeto> {
    let selector = ed
        .get("objeto")
        .and_then(Value::as_str)
        .ok_or_else(|| missing("objeto", "en cada edición"))?;
    let campos = ed
        .get("campos")
        .and_then(Value::as_object)
        .ok_or_else(|| missing("campos", "objeto con los campos a cambiar"))?;
    Ok(EdicionObjeto {
        selector: selector.to_string(),
        cambios: edit::changes_from_json(campos),
    })
}

fn reporte(plan: &[EdicionObjeto], outcomes: &[edit::ChangeOutcome]) -> Vec<Value> {
    plan.iter()
        .scan(0usize, |inicio, ed| {
            let fin = *inicio + ed.cambios.len();
            let changes = outcomes.get(*inicio..fin).unwrap_or_default();
            *inicio = fin;
            Some(json!({
                "objeto": ed.selector,
                "changes": changes.iter().map(edit::ChangeOutcome::to_json).collect::<Vec<_>>(),
            }))
        })
        .collect()
}
