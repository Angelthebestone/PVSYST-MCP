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
    items.extend(crate::projects::lifecycle::variantes::variant_files(
        parent, stem,
    )?);
    let destino = if let Some(d) = opt_str(args, &["destino"]) {
        resolver_ruta(&workspace, d, Alcance::de_args(args))?
    } else {
        parent.join(format!("{stem}.zip"))
    };
    crate::components::sobrescritura::exigir_sobrescribir(&destino, args)?;
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
