//! Reading PVObject structures from files.

use crate::args::flag;
use crate::args::opt_str;
use crate::args::workspace_of;
use crate::config::Paths;
use crate::error::{Error, Result};
use crate::format::edit;
use crate::format::object;
use crate::objects::resolucion::resolve_file;
use serde_json::{json, Value};
use std::fs;
use std::path::Path;

/// Root nodes to return: the whole tree (which must be complete), or just the object
/// named by `objeto` (same selector as editar; a .VCi is ~1.5 MB, so the full tree is ~15k tokens).
fn raices_a_leer(args: &Value, text: &str, origen: &Path) -> Result<Vec<object::PvNode>> {
    let Some(sel) = opt_str(args, &["objeto"]) else {
        return object::parse_completo(text, origen);
    };
    let spans = edit::spans_of(text.as_bytes());
    let sp = edit::find_span(&spans, sel)
        .map(|i| &spans[i])
        .ok_or_else(|| {
            Error::NotFound(format!("Object '{sel}' not found; see listar que:objetos"))
        })?;
    let slice: Vec<&str> = text
        .lines()
        .skip(sp.open)
        .take(sp.close + 1 - sp.open)
        .collect();
    Ok(object::parse(&slice.join("\n")))
}

fn aplanar(prefix: &str, node: &object::PvNode, map: &mut serde_json::Map<String, Value>) {
    for (k, v) in &node.fields {
        let key = if prefix.is_empty() {
            k.clone()
        } else {
            format!("{prefix}.{k}")
        };
        map.insert(key, json!(v));
    }
    for c in &node.children {
        let seg = c.role.clone().unwrap_or_else(|| c.obj_type.clone());
        let p = if prefix.is_empty() {
            seg
        } else {
            format!("{prefix}.{seg}")
        };
        aplanar(&p, c, map);
    }
}

pub(crate) fn leer_objeto(paths: &Paths, args: &Value) -> Result<Value> {
    let workspace = workspace_of(paths, args);
    let path = resolve_file(&workspace, args)?;
    let bytes =
        fs::read(&path).map_err(|e| Error::io(format!("Cannot read {}", path.display()), e))?;
    let text = String::from_utf8_lossy(&bytes);
    object::exigir_no_vacio(&text, &path)?;
    let is_text = object::is_pvobject(&text);

    let mut out = json!({
        "file": path.to_string_lossy(),
        "size_bytes": bytes.len(),
        "is_pvobject": is_text,
    });
    if is_text {
        let roots = raices_a_leer(args, &text, &path)?;
        if flag(args, &["plano"]) {
            let mut flat = serde_json::Map::new();
            for r in &roots {
                aplanar("", r, &mut flat);
            }
            out["flat"] = Value::Object(flat);
        } else {
            out["objects"] = Value::Array(roots.iter().map(object::PvNode::to_json).collect());
        }
    } else {
        out["note"] = json!("Binary file; use leer with base64:true for its bytes.");
    }
    Ok(out)
}
