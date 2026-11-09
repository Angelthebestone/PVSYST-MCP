//! Snapshot of one project's files before a design run, so a failed run can be undone.

use crate::config::Paths;
use crate::error::{Error, Result};
use crate::history;
use std::path::{Path, PathBuf};

/// Files of project `nombre` in the workspace `Projects` folder: empty when the folder
/// does not exist, an error when it cannot be read (a rollback must not guess).
fn archivos_del_proyecto(paths: &Paths, nombre: &str) -> Result<Vec<PathBuf>> {
    let dir = paths.workspace.join("Projects");
    let entradas = match std::fs::read_dir(&dir) {
        Ok(e) => e,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(Vec::new()),
        Err(e) => return Err(Error::io(format!("No se pudo leer {}", dir.display()), e)),
    };
    let mut archivos = Vec::new();
    for entrada in entradas {
        let ruta = entrada
            .map_err(|e| Error::io(format!("No se pudo leer {}", dir.display()), e))?
            .path();
        if es_del_proyecto(&ruta, nombre) {
            archivos.push(ruta);
        }
    }
    Ok(archivos)
}

fn es_del_proyecto(ruta: &Path, nombre: &str) -> bool {
    ruta.file_stem()
        .and_then(|s| s.to_str())
        .is_some_and(|s| s.eq_ignore_ascii_case(nombre))
}

/// The files of one project before a design run, to undo the run if it fails. Only
/// that project's files are touched: other files PVsyst or the user add meanwhile stay.
pub(crate) struct FotoProyecto {
    nombre: String,
    originales: Vec<(PathBuf, Vec<u8>)>,
}

impl FotoProyecto {
    /// Reads the project's current files; fails if the folder or a file cannot be read.
    pub(crate) fn tomar(paths: &Paths, nombre: &str) -> Result<Self> {
        let originales = archivos_del_proyecto(paths, nombre)?
            .into_iter()
            .map(|p| {
                let bytes = crate::workspace::leer_bytes(&p)?;
                Ok((p, bytes))
            })
            .collect::<Result<_>>()?;
        Ok(Self {
            nombre: nombre.to_string(),
            originales,
        })
    }

    /// Removes the project's files created since the photo and puts the originals back.
    /// Best effort: the step error is what the caller reports.
    pub(crate) fn deshacer(&self, paths: &Paths) {
        let actuales = match archivos_del_proyecto(paths, &self.nombre) {
            Ok(a) => a,
            Err(error) => {
                tracing::warn!(%error, "el deshacer no pudo listar el proyecto");
                Vec::new()
            }
        };
        let nuevos = actuales
            .iter()
            .filter(|p| !self.originales.iter().any(|(o, _)| o == *p));
        for nuevo in nuevos {
            if let Err(error) = std::fs::remove_file(nuevo) {
                tracing::warn!(archivo = %nuevo.display(), %error, "el deshacer no pudo borrar un archivo nuevo");
            }
        }
        for (ruta, bytes) in &self.originales {
            if let Err(error) = history::escribir_atomico(ruta, bytes) {
                tracing::warn!(archivo = %ruta.display(), %error, "el deshacer no pudo restaurar un original");
            }
        }
    }
}
