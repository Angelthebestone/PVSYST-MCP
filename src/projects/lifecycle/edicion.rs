//! Variant object and field editing.

use crate::args::missing;
use crate::config::Paths;
use crate::error::Result;
use crate::format::edit;
use crate::history;
use crate::projects::lifecycle::variantes::{abrir_variante, VarianteAbierta};
use serde_json::{json, Value};

/// Apply field edits to a variant's objects.
/// `ediciones`: [{ "objeto": "pvSubArray", "campos": { "NStringCh": 48 } }]
pub(crate) fn editar_variante(paths: &Paths, args: &Value) -> Result<Value> {
    let ediciones = args
        .get("ediciones")
        .and_then(|v| v.as_array())
        .ok_or_else(|| missing("ediciones", "array of {objeto, campos}"))?;

    let VarianteAbierta {
        workspace,
        vpath,
        mut bytes,
        ..
    } = abrir_variante(paths, args)?;

    let mut report = Vec::with_capacity(ediciones.len());
    for ed in ediciones {
        let selector = ed
            .get("objeto")
            .and_then(|v| v.as_str())
            .ok_or_else(|| missing("objeto", "en cada edición"))?;
        let campos = ed
            .get("campos")
            .and_then(|v| v.as_object())
            .ok_or_else(|| missing("campos", "objeto con los campos a cambiar"))?;
        let changes = edit::changes_from_json(campos);
        let (new_bytes, outcomes) = edit::edit_fields(&bytes, selector, &changes)?;
        bytes = new_bytes;
        report.push(json!({
            "objeto": selector,
            "changes": outcomes.iter().map(edit::ChangeOutcome::to_json).collect::<Vec<_>>(),
        }));
    }

    let historial_id = history::reemplazar(&workspace, &vpath, &bytes, "editar_variante")?;
    Ok(json!({
        "variant_file": vpath.to_string_lossy(),
        "historial_id": historial_id,
        "edits": report,
    }))
}
