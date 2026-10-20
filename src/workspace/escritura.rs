//! Atomic file writes with automatic history snapshots.

use crate::args::{flag, req_str, workspace_of};
use crate::config::Paths;
use crate::error::{Error, Result};
use crate::history;
use crate::workspace::base64::decode_base64;
use crate::workspace::rutas::{resolver_ruta, Alcance};
use serde_json::{json, Value};
use std::fs;

pub(crate) fn escribir_archivo(paths: &Paths, args: &Value) -> Result<Value> {
    let workspace = workspace_of(paths, args);
    let p = req_str(args, &["archivo", "file"], "")?;
    let path = resolver_ruta(&workspace, p, Alcance::de_args(args))?;
    let content = req_str(args, &["contenido", "content"], "")?;
    let bytes = if flag(args, &["base64"]) {
        decode_base64(content)?
    } else {
        content.as_bytes().to_vec()
    };

    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .map_err(|e| Error::io(format!("Cannot create {}", parent.display()), e))?;
    }
    let historial_id = if path.is_file() {
        Some(history::snapshot(
            &workspace,
            std::slice::from_ref(&path),
            "escribir_archivo",
        )?)
    } else {
        None
    };
    history::escribir_atomico(&path, &bytes)?;
    Ok(json!({
        "path": path.to_string_lossy(),
        "bytes_written": bytes.len(),
        "historial_id": historial_id,
    }))
}
