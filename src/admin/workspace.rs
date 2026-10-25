//! Workspace lifecycle administration: creation and export.

use crate::admin::archive::zip_into;
use crate::args::{flag, missing, opt_str, workspace_of};
use crate::config::Paths;
use crate::error::{Error, Result};
use crate::workspace::{resolver_ruta, Alcance};
use serde_json::{json, Value};
use std::fs;
use std::path::{Component, Path, PathBuf};

/// Standard workspace sub-folders (per PVsyst documentation).
const WORKSPACE_DIRS: &[&str] = &[
    "ComposPV",
    "DataMeas",
    "Meteo",
    "Models",
    "Projects",
    "Shadings",
    "Other",
    "Sites",
    "UserData",
    "Templates",
    "UserBatch",
    "UserHourly",
    "UserHourlyParams",
    "UserImages",
    "UserOptimization",
    "UserRecycleBin",
    "Results",
];

const COMPOS_SUBDIRS: &[&str] = &[
    "PVModules",
    "Inverters",
    "Batteries",
    "Pumps",
    "Regulators",
    "RegulPmp",
    "Gensets",
    "Sellers",
    "Clients",
];

/// Create a new empty workspace with the standard folder layout.
pub(crate) fn crear_workspace(paths: &Paths, args: &Value) -> Result<Value> {
    let base = args
        .get("ruta")
        .or_else(|| args.get("path"))
        .and_then(Value::as_str)
        .map(PathBuf::from)
        .ok_or_else(|| {
            missing(
                "ruta",
                "destination folder, e.g. C:\\\\Users\\\\me\\\\PVsyst8.1_Data",
            )
        })?;
    if base.components().any(|c| matches!(c, Component::ParentDir)) {
        return Err(Error::Invalid(
            "'ruta' no admite '..': indica una ruta explícita".into(),
        ));
    }
    let base = if Path::new(&base).is_absolute() {
        base
    } else {
        paths
            .workspace
            .parent()
            .unwrap_or(Path::new("."))
            .join(base)
    };

    if base.exists() && !flag(args, &["sobrescribir"]) {
        return Err(Error::Invalid(format!(
            "Workspace already exists: {} (pass sobrescribir=true to reuse)",
            base.display()
        )));
    }
    fs::create_dir_all(&base)
        .map_err(|e| Error::io(format!("Cannot create {}", base.display()), e))?;
    let mut created = Vec::new();
    for d in WORKSPACE_DIRS {
        let p = base.join(d);
        fs::create_dir_all(&p)
            .map_err(|e| Error::io(format!("Cannot create {}", p.display()), e))?;
        created.push((*d).to_string());
    }
    let compos = base.join("ComposPV");
    for d in COMPOS_SUBDIRS {
        let sub = compos.join(d);
        fs::create_dir_all(&sub)
            .map_err(|e| Error::io(format!("Cannot create {}", sub.display()), e))?;
    }
    Ok(json!({
        "workspace": base.to_string_lossy(),
        "folders": created,
        "note": "Data-only workspace: PVsystCLI run-simulation only accepts the workspace configured in PVsyst itself (see PVSYST_CLI_REFERENCE.md section 9)."
    }))
}

/// Export the whole workspace to a zip (documented "Export workspace").
pub(crate) fn exportar_workspace(paths: &Paths, args: &Value) -> Result<Value> {
    let workspace = workspace_of(paths, args);
    if !workspace.exists() {
        return Err(Error::NotFound(format!(
            "Workspace not found: {}",
            workspace.display()
        )));
    }
    let destino = destino_zip(&workspace, args)?;
    if destino.exists() && !flag(args, &["sobrescribir"]) {
        return Err(Error::Invalid(format!(
            "El zip ya existe: {} (pasa sobrescribir:true)",
            destino.display()
        )));
    }
    if let Some(parent) = destino.parent() {
        fs::create_dir_all(parent)
            .map_err(|e| Error::io(format!("Cannot create {}", parent.display()), e))?;
    }
    let names: Vec<std::ffi::OsString> = fs::read_dir(&workspace)
        .map_err(|e| Error::io(format!("Cannot list {}", workspace.display()), e))?
        .flatten()
        .map(|e| e.file_name())
        .collect();
    zip_into(&destino, &workspace, &names)?;
    let size = fs::metadata(&destino)
        .map_err(|e| Error::io(format!("Cannot read metadata of {}", destino.display()), e))?
        .len();
    Ok(
        json!({ "zip": destino.to_string_lossy(), "size_bytes": size,
        "source_workspace": workspace.to_string_lossy() }),
    )
}

/// Destination of the export zip. An explicit `destino` goes through the usual
/// confinement; without it the zip sits next to the workspace, which needs
/// `permitir_fuera_del_workspace` because that is outside the workspace.
fn destino_zip(workspace: &Path, args: &Value) -> Result<PathBuf> {
    if let Some(d) = opt_str(args, &["destino"]) {
        return resolver_ruta(workspace, d, Alcance::de_args(args));
    }
    let nombre = workspace
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("workspace");
    let junto = workspace
        .parent()
        .unwrap_or(Path::new("."))
        .join(format!("{nombre}.zip"));
    if matches!(Alcance::de_args(args), Alcance::Libre) {
        return Ok(junto);
    }
    Err(Error::Invalid(format!(
        "Sin 'destino' el zip se escribiría fuera del workspace ({}): pasa destino o permitir_fuera_del_workspace:true",
        junto.display()
    )))
}
