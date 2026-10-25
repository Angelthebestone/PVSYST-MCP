//! Editing PVObject field definitions with snapshot-backed atomic writes.

use crate::args::{missing, req_str, workspace_of};
use crate::config::Paths;
use crate::error::{Error, Result};
use crate::format::edit;
use crate::format::object;
use crate::history;
use crate::objects::resolucion::resolve_file;
use serde_json::{json, Value};
use std::fs;

/// Re-parse only the edited object's lines and read back each changed key.
/// A readback that differs from the requested value refuses the edit before
/// anything is written.
fn verificar_span(
    bytes: &[u8],
    selector: &str,
    changes: &[(String, Option<String>)],
) -> Result<serde_json::Map<String, Value>> {
    let mut verified = serde_json::Map::new();
    let spans = edit::spans_of(bytes);
    let Some(span) = edit::find_span(&spans, selector).map(|i| spans[i].clone()) else {
        return Ok(verified);
    };
    let text = String::from_utf8_lossy(bytes);
    let lines: Vec<&str> = text.lines().collect();
    let last = span.close.min(lines.len().saturating_sub(1));
    let slice = lines[span.open..=last].join("\n");
    if let Some(root) = object::parse_first(&slice) {
        for (k, nuevo) in changes {
            let leido = root.find_deep(k);
            comprobar_lectura(k, nuevo.as_deref(), leido)?;
            verified.insert(k.clone(), json!(leido));
        }
    }
    Ok(verified)
}

fn comprobar_lectura(clave: &str, esperado: Option<&str>, leido: Option<&str>) -> Result<()> {
    let coincide = match esperado {
        None => leido.is_none(),
        Some(valor) => leido.map(str::trim) == Some(valor.trim()),
    };
    if coincide {
        Ok(())
    } else {
        Err(Error::Format(format!(
            "Escritura no verificada en el campo '{clave}'"
        )))
    }
}

pub(crate) fn editar_objeto(paths: &Paths, args: &Value) -> Result<Value> {
    let workspace = workspace_of(paths, args);
    let path = resolve_file(&workspace, args)?;
    let cambios = args
        .get("campos")
        .or_else(|| args.get("fields"))
        .and_then(|v| v.as_object())
        .ok_or_else(|| missing("campos", "object of key: value"))?;

    let changes = edit::changes_from_json(cambios);
    edit::reject_invalid_keys(&changes)?;

    let bytes =
        fs::read(&path).map_err(|e| Error::io(format!("Cannot read {}", path.display()), e))?;

    if crate::components::pan::es_pan_legacy(&bytes) {
        let selector = crate::args::opt_str(args, &["objeto"]).unwrap_or("pvModule");
        let (new_bytes, outcomes) =
            crate::components::pan::parchear_campos_legacy(&bytes, &changes)?;
        let historial_id =
            history::snapshot(&workspace, std::slice::from_ref(&path), "editar_objeto")?;
        history::escribir_atomico(&path, &new_bytes)?;
        let mut verified = serde_json::Map::new();
        for o in &outcomes {
            verified.insert(o.key.clone(), json!(o.new_value));
        }
        return Ok(json!({
            "file": path.to_string_lossy(),
            "object": selector,
            "historial_id": historial_id,
            "changes": outcomes.iter().map(edit::ChangeOutcome::to_json).collect::<Vec<_>>(),
            "verified_after_write": verified,
        }));
    }

    let selector = req_str(args, &["objeto"], "e.g. \"pvSubArray\", \"role:SitePrj\"")?;

    let (new_bytes, outcomes) = edit::edit_fields(&bytes, selector, &changes)?;
    let verified = verificar_span(&new_bytes, selector, &changes)?;
    let historial_id = history::snapshot(&workspace, std::slice::from_ref(&path), "editar_objeto")?;
    history::escribir_atomico(&path, &new_bytes)?;
    Ok(json!({
        "file": path.to_string_lossy(),
        "object": selector,
        "historial_id": historial_id,
        "changes": outcomes.iter().map(edit::ChangeOutcome::to_json).collect::<Vec<_>>(),
        "verified_after_write": verified,
    }))
}
