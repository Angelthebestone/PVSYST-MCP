//! Listing PVObject elements and hierarchical span structure in a file.

use crate::args::workspace_of;
use crate::config::Paths;
use crate::error::{Error, Result};
use crate::format::edit;
use crate::format::object;
use crate::objects::resolucion::resolve_file;
use serde_json::{json, Value};
use std::fs;

pub(crate) fn listar_objetos(paths: &Paths, args: &Value) -> Result<Value> {
    let workspace = workspace_of(paths, args);
    let path = resolve_file(&workspace, args)?;
    let bytes =
        fs::read(&path).map_err(|e| Error::io(format!("Cannot read {}", path.display()), e))?;
    let text = String::from_utf8_lossy(&bytes);
    if !object::is_pvobject(&text) {
        return Err(Error::Format(format!(
            "{} is not a PVObject text file",
            path.display()
        )));
    }
    let spans = edit::spans_of(&bytes);
    let mut out: Vec<Value> = Vec::with_capacity(spans.len());
    for (i, s) in spans.iter().enumerate() {
        let depth = spans
            .iter()
            .enumerate()
            .filter(|(j, o)| *j != i && o.open < s.open && o.close > s.close)
            .count();
        out.push(json!({
            "type": s.ty,
            "role": s.role,
            "depth": depth,
            "line_open": s.open + 1,
            "line_close": s.close + 1,
        }));
    }
    Ok(json!({ "file": path.to_string_lossy(), "count": out.len(), "objects": out }))
}
