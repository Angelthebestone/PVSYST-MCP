//! Adds whole `PVObject` blocks to a project variant: the `proyecto anadir`
//! element catalogue and the generic `editar insertar_objeto` / `eliminar_objeto` file operations.

pub(crate) mod baterias;
pub(crate) mod catalogo;
pub(crate) mod horizonte;
pub(crate) mod objetos;

use crate::error::{Error, Result};
use crate::history;
use std::path::{Path, PathBuf};

pub(crate) use catalogo::anadir;
pub(crate) use objetos::{eliminar_objeto_archivo, insertar_objeto_archivo};

/// Snapshots `path` into the workspace history, then writes `nuevo` atomically.
/// Returns the snapshot id.
pub(crate) fn escribir_con_historial(
    ws: &Path,
    path: &Path,
    nuevo: &[u8],
    motivo: &str,
) -> Result<String> {
    let id = history::snapshot(ws, &[path.to_path_buf()], motivo)?;
    history::escribir_atomico(path, nuevo)?;
    Ok(id)
}

/// Variant file `<proyecto stem>.<variante>` next to its PRJ.
pub(crate) fn ruta_variante(prj: &Path, variante: &str) -> Result<PathBuf> {
    let stem = crate::projects::lifecycle::variantes::stem_de(prj)?;
    let parent = prj.parent().unwrap_or_else(|| Path::new(""));
    let destino = crate::projects::lifecycle::variantes::ruta_variante(parent, &stem, variante)?;
    if destino.is_file() {
        Ok(destino)
    } else {
        Err(Error::NotFound(format!(
            "Variante '{variante}' no encontrada junto a {}",
            prj.display()
        )))
    }
}
