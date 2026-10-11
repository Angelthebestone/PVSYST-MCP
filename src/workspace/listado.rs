//! Safe directory listings without following symlink cycles, enforcing depth and count bounds.

use crate::args::workspace_of;
use crate::config::Paths;
use crate::error::{Error, Result};
use crate::workspace::rutas::{resolver_ruta, Alcance};
use serde_json::{json, Value};
use std::fs;
use std::path::{Path, PathBuf};
use std::time::SystemTime;

use super::recorrido::{extraer_opciones_listado, recorrer};

/// Maximum recursion depth allowed during directory traversals.
pub(crate) const MAX_PROFUNDIDAD: usize = 32;

/// Maximum files collected in a single directory listing.
pub(crate) const MAX_ARCHIVOS: usize = 10_000;

#[derive(Debug, Default)]
pub(crate) struct InfoRecorrido {
    pub(crate) archivos: Vec<PathBuf>,
    pub(crate) errores: Vec<String>,
    pub(crate) truncado: bool,
}

#[must_use]
#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub(crate) struct ResumenRecorrido {
    pub(crate) truncado: bool,
    pub(crate) errores: Vec<String>,
}

impl ResumenRecorrido {
    /// Human-readable warnings; empty when the inventory is complete.
    pub(crate) fn advertencias(&self) -> Vec<String> {
        let mut avisos: Vec<String> = self
            .errores
            .iter()
            .map(|e| format!("No se pudo leer {e}"))
            .collect();
        if self.truncado {
            avisos.push(format!(
                "Inventario incompleto: límite de {MAX_ARCHIVOS} archivos o {MAX_PROFUNDIDAD} niveles"
            ));
        }
        avisos
    }
}

pub(crate) fn mtime_secs(p: &Path) -> u64 {
    fs::metadata(p)
        .ok()
        .and_then(|m| m.modified().ok())
        .and_then(|t| t.duration_since(SystemTime::UNIX_EPOCH).ok())
        .map_or(0, |d| d.as_secs())
}

fn serializar_items(archivos: &[PathBuf], ext: &Option<String>, workspace: &Path) -> Vec<Value> {
    let mut items: Vec<Value> = archivos
        .iter()
        .filter(|f| {
            ext.as_ref().is_none_or(|want| {
                f.extension()
                    .and_then(|e| e.to_str())
                    .is_some_and(|e| format!(".{}", e.to_lowercase()) == *want)
            })
        })
        .map(|f| {
            json!({
                "relative": f.strip_prefix(workspace).unwrap_or(f).to_string_lossy(),
                "size_bytes": fs::metadata(f).map_or(0, |m| m.len()),
                "modified_unix": mtime_secs(f),
            })
        })
        .collect();
    items.sort_by(|a, b| a["relative"].as_str().cmp(&b["relative"].as_str()));
    items
}

pub(crate) fn listar_archivos(paths: &Paths, args: &Value) -> Result<Value> {
    let workspace = workspace_of(paths, args);
    let folder = args
        .get("carpeta")
        .or_else(|| args.get("folder"))
        .and_then(|v| v.as_str())
        .unwrap_or(".");
    let ext = args.get("extension").and_then(|v| v.as_str()).map(|e| {
        let e = e.trim_start_matches('.').to_lowercase();
        format!(".{e}")
    });
    let root = resolver_ruta(&workspace, folder, Alcance::de_args(args))?;
    if !root.exists() {
        return Err(Error::NotFound(format!(
            "Folder not found: {}",
            root.display()
        )));
    }
    let opciones = extraer_opciones_listado(args);
    let mut info = InfoRecorrido::default();
    recorrer(&root, &opciones, &mut info);

    let items = serializar_items(&info.archivos, &ext, &workspace);
    let mut resp = json!({
        "workspace": workspace.to_string_lossy(),
        "folder": root.to_string_lossy(),
        "extension": ext,
        "count": items.len(),
        "files": items,
        "truncado": info.truncado,
    });
    if !info.errores.is_empty() {
        resp["errores"] = json!(info.errores);
    }
    Ok(resp)
}
