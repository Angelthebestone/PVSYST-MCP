//! Administration tools: licensing, logs, workspace lifecycle,
//! project export and component transfers.

pub(crate) mod archive;
pub(crate) mod componentes;
pub(crate) mod importacion;
pub(crate) mod licencia;
pub(crate) mod logs;
pub(crate) mod proyecto;
pub(crate) mod workspace;

/// Entries of `dir`. An entry that cannot be read is an error: dropping it would
/// make an import or export look complete while files were left out.
pub(crate) fn entradas_de(dir: &std::path::Path) -> crate::error::Result<Vec<std::path::PathBuf>> {
    let error = |e| crate::error::Error::io(format!("No se pudo listar {}", dir.display()), e);
    std::fs::read_dir(dir)
        .map_err(error)?
        .map(|e| e.map(|e| e.path()).map_err(error))
        .collect()
}

#[cfg(test)]
#[path = "../../tests/unit/admin/tests.rs"]
mod tests;
#[cfg(test)]
#[path = "../../tests/unit/admin/exportar.rs"]
mod tests_exportar;

pub(crate) use componentes::{exportar_componente, importar_componente};
pub(crate) use importacion::importar_workspace;
pub(crate) use licencia::licencia;
pub(crate) use logs::exportar_logs;
pub(crate) use proyecto::exportar_proyecto;
pub(crate) use workspace::{crear_workspace, exportar_workspace};
