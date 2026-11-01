//! Path confinement and resolution within the workspace.

use crate::args::flag;
pub(crate) use crate::config::Alcance;
use crate::error::{Error, Result};
use serde_json::Value;
use std::fs;
use std::path::{Component, Path, PathBuf};

/// Argument that lets a caller reach paths outside the workspace.
pub(crate) const PERMITIR_FUERA: &str = "permitir_fuera_del_workspace";

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
        .map_err(|e| Error::io(format!("Workspace no accesible {}", base.display()), e))?;
    let mut ancestro = joined.as_path();
    // `symlink_metadata`, not `exists`: a dangling link must be canonicalised (and
    // fail) instead of being skipped, or a write through it would land outside.
    while fs::symlink_metadata(ancestro).is_err() {
        ancestro = ancestro
            .parent()
            .ok_or_else(|| Error::Invalid(format!("Ruta sin ancestro existente: '{rel}'")))?;
    }
    let real = ancestro
        .canonicalize()
        .map_err(|e| Error::io(format!("No se pudo resolver {}", ancestro.display()), e))?;
    if !real.starts_with(&base_c) {
        return Err(Error::Invalid(format!("Ruta fuera del workspace: '{rel}'")));
    }
    Ok(joined)
}

/// Is `p` a plain relative path (only named components and `.`)?
///
/// A rooted path without drive (`\\Users\\x`), a drive-relative one (`C:x`) or one with
/// `..` is not: `Path::join` would replace or leave the base, so those must go
/// through [`resolver_ruta`].
pub(crate) fn es_relativa_simple(p: &Path) -> bool {
    p.components()
        .all(|c| matches!(c, Component::Normal(_) | Component::CurDir))
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
/// First existing file among `candidatos`, each resolved under the workspace with
/// `resolver_ruta`. A candidate that escapes the workspace is an error, not a miss:
/// the caller learns why instead of reading "not found". O(c) stats.
pub(crate) fn primer_existente_en_workspace(
    workspace: &Path,
    candidatos: &[String],
    alcance: Alcance,
) -> Result<Option<PathBuf>> {
    for c in candidatos {
        match resolver_ruta(workspace, c, alcance) {
            Ok(p) if p.is_file() => return Ok(Some(p)),
            Ok(_) => {}
            Err(e @ Error::Invalid(_)) => return Err(e),
            Err(e) => tracing::debug!(candidato = %c, error = %e, "candidato no resoluble"),
        }
    }
    Ok(None)
}

/// First existing file `<base>/<nombre>` over `bases` × `nombres`, each confined to
/// its base with `ruta_segura`: read-only folders (DataRO) cannot be left with `..`
/// or an absolute name. A base that does not exist is skipped. O(b·n) stats.
pub(crate) fn primer_existente_en_bases(bases: &[PathBuf], nombres: &[String]) -> Option<PathBuf> {
    bases.iter().find_map(|base| {
        nombres
            .iter()
            .filter_map(|n| ruta_segura(base, n).ok())
            .find(|p| p.is_file())
    })
}

/// Specification for resolving a file with natural folder and DataRO fallbacks.
#[derive(Debug, Clone)]
pub(crate) struct PeticionResolucion<'a> {
    pub(crate) nombre: &'a str,
    pub(crate) subcarpeta_ws: Option<&'a str>,
    pub(crate) rutas_dataro: Vec<PathBuf>,
}

/// Resolves a file requested by a tool, respecting workspace confinement.
pub(crate) fn resolver_archivo_con_fallback(
    workspace: &Path,
    args: &Value,
    peticion: &PeticionResolucion<'_>,
) -> Result<PathBuf> {
    let nombre = peticion.nombre.trim();
    if nombre.is_empty() {
        return Err(Error::Invalid(
            "Nombre de archivo no puede ser vacío".into(),
        ));
    }
    let tiene_separadores = nombre.contains('/') || nombre.contains('\\');
    let es_absoluto = Path::new(nombre).is_absolute();
    if tiene_separadores || es_absoluto {
        let resuelta = resolver_ruta(workspace, nombre, Alcance::de_args(args))?;
        if resuelta.exists() {
            return Ok(resuelta);
        }
        return Err(Error::NotFound(format!("Archivo no encontrado: {nombre}")));
    }
    crate::workspace::validar_nombre(nombre)?;
    if let Some(sub) = peticion.subcarpeta_ws {
        let sub_dir = workspace.join(sub);
        if let Ok(en_sub) = ruta_segura(&sub_dir, nombre) {
            if en_sub.exists() {
                return Ok(en_sub);
            }
        }
    }
    if let Ok(en_raiz) = ruta_segura(workspace, nombre) {
        if en_raiz.exists() {
            return Ok(en_raiz);
        }
    }
    for ro in &peticion.rutas_dataro {
        if ro.exists() {
            return Ok(ro.clone());
        }
    }
    Err(Error::NotFound(format!("Archivo no encontrado: {nombre}")))
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

#[cfg(test)]
#[path = "../../tests/unit/workspace/rutas.rs"]
mod tests;
