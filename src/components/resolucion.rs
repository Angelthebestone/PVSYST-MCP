//! Resolve a component reference (bare name, stem or path) to a file.
//!
//! Bare names are looked up in the workspace `ComposPV` folders and the PVsyst
//! read-only database; any path the caller spells out goes through the common
//! scope policy (`resolver_ruta`), except paths inside the read-only database,
//! which are always readable and never mutated.

use super::{compos_dir, folder_for};
use crate::args::no_encontrado;
use crate::config::compos_ro;
use crate::error::{Error, Result};
use crate::workspace::{resolver_ruta, Alcance};
use std::fs;
use std::path::{Path, PathBuf};

/// Largest component file loaded into memory (real PAN/OND files are < 1 MB).
const MAX_BYTES_COMPONENTE: u64 = 16 * 1024 * 1024;

/// What to resolve and under which path scope.
pub(crate) struct ConsultaComponente<'a> {
    /// Component kind (`PAN`, `OND`, ...).
    pub(crate) kind: &'a str,
    /// Bare file name, stem, or a path relative to the workspace / absolute.
    pub(crate) name: &'a str,
    pub(crate) alcance: Alcance,
}

impl<'a> ConsultaComponente<'a> {
    /// A component name stored inside a PVsyst file (variant, project): always
    /// resolved within the workspace and the read-only database.
    pub(crate) fn referencia(kind: &'a str, name: &'a str) -> Self {
        Self {
            kind,
            name,
            alcance: Alcance::Workspace,
        }
    }
}

pub(crate) fn resolve_component(
    workspace: &Path,
    consulta: &ConsultaComponente<'_>,
) -> Result<PathBuf> {
    let direct = Path::new(consulta.name);
    if direct.is_absolute() || direct.components().count() > 1 {
        return ruta_explicita(workspace, consulta);
    }
    let folder = folder_for(consulta.kind);
    let mut candidates: Vec<PathBuf> = Vec::new();
    if let Some(f) = folder {
        candidates.push(compos_dir(workspace, f).join(direct));
        candidates.push(compos_ro().join(f).join(direct));
    }
    candidates.push(workspace.join(direct));
    // Accept the bare stem too ("Generic_50kW" -> "Generic_50kW.OND").
    if direct.extension().is_none() {
        let ext = consulta.kind.trim_start_matches('.').to_uppercase();
        let with_ext: Vec<PathBuf> = candidates.iter().map(|c| c.with_extension(&ext)).collect();
        candidates.extend(with_ext);
    }
    candidates.into_iter().find(|c| c.is_file()).ok_or_else(|| {
        let carpeta = folder.map_or_else(|| workspace.to_path_buf(), |f| compos_dir(workspace, f));
        let opciones = componentes_en(&carpeta, consulta.kind);
        no_encontrado(
            "Componente",
            consulta.name,
            &opciones,
            &carpeta.to_string_lossy(),
        )
    })
}

/// A path the caller spelled out: inside the read-only database, or under the scope policy.
fn ruta_explicita(workspace: &Path, consulta: &ConsultaComponente<'_>) -> Result<PathBuf> {
    let direct = Path::new(consulta.name);
    let ruta = if direct.is_absolute() && es_base_solo_lectura(direct) {
        direct.to_path_buf()
    } else {
        resolver_ruta(workspace, consulta.name, consulta.alcance)?
    };
    if ruta.is_file() {
        return Ok(ruta);
    }
    Err(Error::NotFound(format!(
        "Componente no encontrado: {}",
        ruta.display()
    )))
}

/// True for files inside PVsyst's read-only component database (compared canonically).
pub(crate) fn es_base_solo_lectura(path: &Path) -> bool {
    match (path.canonicalize(), compos_ro().canonicalize()) {
        (Ok(real), Ok(base)) => real.starts_with(base),
        _ => false,
    }
}

/// Component bytes, refusing files too large to be a PVsyst component.
pub(crate) fn leer_bytes_componente(path: &Path) -> Result<Vec<u8>> {
    let error = |e| Error::io(format!("Cannot read {}", path.display()), e);
    let tamano = fs::metadata(path).map_err(error)?.len();
    if tamano > MAX_BYTES_COMPONENTE {
        return Err(Error::Invalid(format!(
            "{} ocupa {tamano} bytes; un componente admite como máximo {MAX_BYTES_COMPONENTE}",
            path.display()
        )));
    }
    fs::read(path).map_err(error)
}

/// File names of `kind` inside `carpeta`, for "did you mean" hints.
fn componentes_en(carpeta: &Path, kind: &str) -> Vec<String> {
    let ext = kind.trim_start_matches('.').to_uppercase();
    fs::read_dir(carpeta)
        .into_iter()
        .flatten()
        .flatten()
        .map(|e| e.path())
        .filter(|p| {
            p.extension()
                .and_then(|s| s.to_str())
                .is_some_and(|s| s.eq_ignore_ascii_case(&ext))
        })
        .filter_map(|p| p.file_name().and_then(|s| s.to_str()).map(str::to_string))
        .collect()
}

/// Files in `dir` with extension `ext` (lowercase, no dot), sorted.
pub(crate) fn files_with_ext(dir: &Path, ext: &str) -> Vec<PathBuf> {
    let Ok(entries) = fs::read_dir(dir) else {
        return Vec::new();
    };
    let mut found: Vec<PathBuf> = entries
        .flatten()
        .map(|e| e.path())
        .filter(|p| {
            p.is_file()
                && p.extension()
                    .and_then(|x| x.to_str())
                    .is_some_and(|x| x.eq_ignore_ascii_case(ext))
        })
        .collect();
    found.sort();
    found
}
