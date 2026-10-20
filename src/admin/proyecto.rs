//! Project export to archive files.

use crate::admin::archive::zip_into;
use crate::args::{opt_str, req_str, workspace_of};
use crate::config::Paths;
use crate::error::{Error, Result};
use crate::projects;
use crate::workspace::{resolver_ruta, Alcance};
use serde_json::{json, Value};
use std::fs;
use std::path::PathBuf;

/// True for variant extensions (`VC?` / `CM?`, case-insensitive).
pub(crate) fn es_extension_variante(ext: &str) -> bool {
    let up = ext.to_ascii_uppercase();
    up.len() == 3 && (up.starts_with("VC") || up.starts_with("CM"))
}

/// Export a project (PRJ + all its variants) to a zip.
pub(crate) fn exportar_proyecto(paths: &Paths, args: &Value) -> Result<Value> {
    let workspace = workspace_of(paths, args);
    let proyecto = req_str(args, &["proyecto"], "project name")?;
    let prj = projects::resolve_project_file(&workspace, proyecto)?;
    let stem = prj
        .file_stem()
        .and_then(|s| s.to_str())
        .ok_or_else(|| Error::Invalid("Invalid name".into()))?;
    let parent = prj
        .parent()
        .ok_or_else(|| Error::Invalid("No parent".into()))?;

    // Only the PRJ and its variant files belong to the project.
    let mut items: Vec<PathBuf> = vec![prj.clone()];
    let entries = fs::read_dir(parent)
        .map_err(|e| Error::io(format!("Cannot read directory {}", parent.display()), e))?;
    for e in entries.flatten() {
        let p = e.path();
        let es_variante = p
            .extension()
            .and_then(|x| x.to_str())
            .is_some_and(es_extension_variante);
        if p.file_stem().and_then(|s| s.to_str()) == Some(stem) && p != prj && es_variante {
            items.push(p);
        }
    }
    let destino = if let Some(d) = opt_str(args, &["destino"]) {
        resolver_ruta(&workspace, d, Alcance::de_args(args))?
    } else {
        parent.join(format!("{stem}.zip"))
    };
    if let Some(p) = destino.parent() {
        fs::create_dir_all(p)
            .map_err(|e| Error::io(format!("Cannot create {}", p.display()), e))?;
    }
    let names: Vec<std::ffi::OsString> = items
        .iter()
        .filter_map(|p| p.file_name().map(std::ffi::OsStr::to_os_string))
        .collect();
    zip_into(&destino, parent, &names)?;
    Ok(json!({
        "zip": destino.to_string_lossy(),
        "files": items.iter().map(|p| p.file_name().map(|n| n.to_string_lossy().into_owned()).unwrap_or_default()).collect::<Vec<_>>(),
        "count": items.len(),
    }))
}
