//! Adds whole `PVObject` blocks to a project variant: the `proyecto anadir`
//! element catalogue and the generic `editar insertar_objeto` / `eliminar_objeto` file operations.

pub(crate) mod baterias;
pub(crate) mod catalogo;
pub(crate) mod horizonte;
pub(crate) mod objetos;

use crate::error::{Error, Result};
use crate::history;
pub(crate) use crate::workspace::leer_bytes;
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
    let stem = prj.file_stem().and_then(|s| s.to_str()).ok_or_else(|| {
        Error::Invalid(format!("Nombre de proyecto no válido: {}", prj.display()))
    })?;
    let destino = prj.with_file_name(format!("{stem}.{variante}"));
    if destino.is_file() {
        Ok(destino)
    } else {
        Err(Error::NotFound(format!(
            "Variante '{variante}' no encontrada junto a {}",
            prj.display()
        )))
    }
}
