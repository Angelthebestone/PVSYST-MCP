//! Resolution of a project name (`X`, `X.PRJ`, `Projects/X.VC0`) to its file in the workspace.

use crate::args::no_encontrado;
use crate::error::{Error, Result};
use crate::workspace::{es_relativa_simple, resolver_ruta, walk_ext, Alcance};
use std::path::{Path, PathBuf};

/// Names of the projects `Projects/` holds, so a "not found" can name them.
///
/// A read-only or unreadable folder lists none: this only feeds a message.
fn proyectos_disponibles(projects_dir: &Path) -> Vec<String> {
    let mut prjs = Vec::new();
    let _ = walk_ext(projects_dir, ".prj", &mut prjs);
    prjs.iter()
        .filter_map(|p| p.file_stem().and_then(|s| s.to_str()).map(str::to_string))
        .collect()
}

/// Accepts `Projects\X.VC0`, `Projects/X.PRJ` and `X.PRJ`: returns the name relative to the Projects folder.
fn relativo_a_projects(name: &str) -> String {
    let normal = name.replace('\\', "/");
    let prefijo = "Projects/";
    match normal.get(..prefijo.len()) {
        Some(inicio) if inicio.eq_ignore_ascii_case(prefijo) => normal[prefijo.len()..].to_string(),
        _ => normal,
    }
}

/// Resolve a project name (with or without extension) to an existing file within the workspace.
pub(crate) fn resolve_project_file(workspace: &Path, name: &str) -> Result<PathBuf> {
    resolve_project_file_con_alcance(workspace, name, Alcance::Workspace)
}

/// Resolve project file with confinement according to the given scope.
pub(crate) fn resolve_project_file_con_alcance(
    workspace: &Path,
    name: &str,
    alcance: Alcance,
) -> Result<PathBuf> {
    if !es_relativa_simple(Path::new(name)) {
        let safe = resolver_ruta(workspace, name, alcance)?;
        if safe.exists() {
            return Ok(safe);
        }
        let projects_dir = workspace.join("Projects");
        return Err(no_encontrado(
            "Proyecto",
            name,
            &proyectos_disponibles(&projects_dir),
            &projects_dir.to_string_lossy(),
        ));
    }
    let projects_dir = workspace.join("Projects");
    let relativo = relativo_a_projects(name);
    let candidates = [
        relativo.clone(),
        format!("{relativo}.PRJ"),
        format!("{relativo}.prj"),
    ];
    for c in &candidates {
        let p = projects_dir.join(c);
        if p.exists() {
            return Ok(p);
        }
    }
    // Recursive search by stem.
    let mut prjs = Vec::new();
    let avisos = walk_ext(&projects_dir, ".prj", &mut prjs).advertencias();
    for p in &prjs {
        if p.file_stem().and_then(|s| s.to_str()) == Some(relativo.as_str()) {
            return Ok(p.clone());
        }
    }
    if !avisos.is_empty() {
        return Err(Error::NotFound(format!(
            "Proyecto '{name}' no encontrado y la búsqueda quedó incompleta: {}",
            avisos.join("; ")
        )));
    }
    let opciones: Vec<String> = prjs
        .iter()
        .filter_map(|p| p.file_stem().and_then(|s| s.to_str()).map(str::to_string))
        .collect();
    Err(no_encontrado(
        "Proyecto",
        name,
        &opciones,
        &projects_dir.to_string_lossy(),
    ))
}
