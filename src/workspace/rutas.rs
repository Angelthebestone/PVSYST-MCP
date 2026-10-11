//! Path confinement and resolution within the workspace.

use crate::args::flag;
use crate::error::{Error, Result};
use serde_json::Value;
use std::fs;
use std::path::{Component, Path, PathBuf};

/// Argument that lets a caller reach paths outside the workspace.
pub(crate) const PERMITIR_FUERA: &str = "permitir_fuera_del_workspace";

/// Where a caller-supplied path may point.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Alcance {
    /// Confined to the workspace (default): absolute paths outside it and `..` are rejected.
    Workspace,
    /// Any path the process can reach; requires an explicit opt-in from the caller.
    Libre,
}

impl Alcance {
    /// `permitir_fuera_del_workspace: true` opts out of confinement; anything else confines.
    pub(crate) fn de_args(args: &Value) -> Self {
        if flag(args, &[PERMITIR_FUERA]) {
            Self::Libre
        } else {
            Self::Workspace
        }
    }
}

/// Join `rel` to `base`, refusing any path that escapes `base`.
///
/// Rejects `..` components outright and checks that the nearest existing
/// ancestor of the result, once canonicalised, stays inside `base`. This works
/// for paths that do not exist yet, which `canonicalize` alone cannot handle.
pub(crate) fn ruta_segura(base: &Path, rel: &str) -> Result<PathBuf> {
    let cand = Path::new(rel);
    if cand.components().any(|c| matches!(c, Component::ParentDir)) {
        return Err(Error::Invalid(format!(
            "Ruta fuera del workspace: '{rel}' contiene '..'"
        )));
    }
    let joined = base.join(cand);
    let base_c = base
        .canonicalize()
        .map_err(|e| Error::Invalid(format!("Workspace no accesible {}: {e}", base.display())))?;
    let mut ancestro = joined.as_path();
    while !ancestro.exists() {
        ancestro = ancestro
            .parent()
            .ok_or_else(|| Error::Invalid(format!("Ruta sin ancestro existente: '{rel}'")))?;
    }
    let real = ancestro
        .canonicalize()
        .map_err(|e| Error::Invalid(format!("No se pudo resolver {}: {e}", ancestro.display())))?;
    if !real.starts_with(&base_c) {
        return Err(Error::Invalid(format!("Ruta fuera del workspace: '{rel}'")));
    }
    Ok(joined)
}

/// Absolute stays, relative is joined to the workspace. Unchecked: use `resolver_ruta`.
fn resolve(workspace: &Path, p: &str) -> PathBuf {
    let cand = Path::new(p);
    if cand.is_absolute() {
        cand.to_path_buf()
    } else {
        workspace.join(cand)
    }
}

/// Resolve a caller path under the given scope: relative paths join the
/// workspace; `Alcance::Workspace` refuses anything that escapes it.
pub(crate) fn resolver_ruta(workspace: &Path, p: &str, alcance: Alcance) -> Result<PathBuf> {
    match alcance {
        Alcance::Workspace => ruta_segura(workspace, p),
        Alcance::Libre => Ok(resolve(workspace, p)),
    }
}

/// Move `origen` into `<workspace>/UserRecycleBin`, keeping its file name.
///
/// A name clash appends `_<YYYYMMDD_HHMMSS>` to the stem. Rename errors are
/// returned to the caller, never swallowed.
pub(crate) fn mover_a_papelera(workspace: &Path, origen: &Path) -> Result<PathBuf> {
    let dir = workspace.join("UserRecycleBin");
    fs::create_dir_all(&dir)
        .map_err(|e| Error::io(format!("No se pudo crear {}", dir.display()), e))?;
    let nombre = origen
        .file_name()
        .and_then(|n| n.to_str())
        .ok_or_else(|| Error::Invalid(format!("Nombre no válido: {}", origen.display())))?;
    let mut destino = dir.join(nombre);
    if destino.exists() {
        let sello = crate::history::sello_compacto();
        let stem = origen
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or(nombre);
        let nuevo = match origen.extension().and_then(|e| e.to_str()) {
            Some(ext) => format!("{stem}_{sello}.{ext}"),
            None => format!("{stem}_{sello}"),
        };
        destino = dir.join(nuevo);
    }
    fs::rename(origen, &destino).map_err(|e| {
        Error::io(
            format!("No se pudo mover {} a la papelera", origen.display()),
            e,
        )
    })?;
    Ok(destino)
}
