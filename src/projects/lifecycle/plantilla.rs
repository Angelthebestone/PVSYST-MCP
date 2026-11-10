//! Where a new project comes from: the template project, the site it is asked to
//! take, and the folder the copy is written to.

use crate::error::Result;
use crate::workspace::Alcance;
use serde_json::Value;
use std::path::{Path, PathBuf};

/// Resolves the base project: the workspace first, then the read-only DataRO demo projects.
pub(super) fn resolver_plantilla(
    workspace: &Path,
    plantilla: &str,
    alcance: Alcance,
) -> Result<PathBuf> {
    match crate::projects::resolve_project_file_con_alcance(workspace, plantilla, alcance) {
        Ok(p) => Ok(p),
        Err(e) => plantilla_demo_ro(plantilla).ok_or(e),
    }
}

/// A PVsyst demo project in DataRO named `plantilla`, with or without `.PRJ`.
fn plantilla_demo_ro(plantilla: &str) -> Option<PathBuf> {
    if !crate::workspace::es_relativa_simple(Path::new(plantilla)) {
        return None;
    }
    let base = crate::config::proyectos_demo_ro();
    [plantilla.to_string(), format!("{plantilla}.PRJ")]
        .iter()
        .map(|c| base.join(c))
        .find(|p| p.is_file())
}

/// Output always goes to the workspace: a DataRO template is never written beside.
pub(super) fn carpeta_de_salida(workspace: &Path, src_prj: &Path, src_parent: &Path) -> PathBuf {
    if src_prj.starts_with(workspace) {
        src_parent.to_path_buf()
    } else {
        workspace.join("Projects")
    }
}

/// The workspace `Sites/<sitio>.SIT` requested with `sitio`; a missing site is an error.
pub(super) fn sit_pedido(workspace: &Path, args: &Value) -> Result<Option<PathBuf>> {
    let Some(sitio) = args.get("sitio").and_then(Value::as_str) else {
        return Ok(None);
    };
    crate::workspace::validar_nombre(sitio)?;
    let dir = workspace.join("Sites");
    let sit = dir.join(format!("{sitio}.SIT"));
    if !sit.is_file() {
        return Err(crate::args::no_encontrado(
            crate::args::SITIO,
            sitio,
            &[],
            &dir.to_string_lossy(),
        ));
    }
    Ok(Some(sit))
}

/// Variant the new project starts from: `variante`, `VC0` by default.
pub(super) fn variante_pedida(args: &Value) -> String {
    args.get("variante")
        .and_then(Value::as_str)
        .unwrap_or("VC0")
        .to_uppercase()
}
