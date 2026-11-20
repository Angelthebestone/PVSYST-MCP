//! Publication: every extracted file is moved into the destination and each
//! target it replaces is backed up first, so a failure can be undone.

use super::error_de_publicacion;
use super::reversion::detalle_reversion;
use super::NO_CREAR;
use crate::error::{Error, Result};
use crate::idioma::{Idioma, Texto};
use std::fs;
use std::path::{Path, PathBuf};

const RUTA_FUERA_TEMPORAL: Texto = Texto {
    es: "Ruta extraída fuera del temporal: {e}",
    en: "Extracted path outside the temporary folder: {e}",
};
const IMPORTACION_CANCELADA: Texto = Texto {
    es: "Importación cancelada en {destino}: {e}; {reversion}",
    en: "Import cancelled at {destino}: {e}; {reversion}",
};
const NO_RESPALDAR: Texto = Texto {
    es: "No se pudo respaldar {ruta}",
    en: "Could not back up {ruta}",
};

fn recolectar(
    dir: &Path,
    directorios: &mut Vec<PathBuf>,
    archivos: &mut Vec<PathBuf>,
) -> Result<()> {
    let mut entradas = crate::workspace::entradas_de(dir)?;
    entradas.sort();
    for p in entradas {
        if p.is_dir() {
            directorios.push(p.clone());
            recolectar(&p, directorios, archivos)?;
        } else if p.is_file() {
            archivos.push(p);
        }
    }
    Ok(())
}

/// Move every extracted file into `destino`, backing up replaced targets in
/// `bak`. On failure everything already published is put back.
pub(super) fn publicar_extraccion(tmp: &Path, destino: &Path, bak: &Path) -> Result<Vec<PathBuf>> {
    let mut directorios = Vec::new();
    let mut archivos = Vec::new();
    recolectar(tmp, &mut directorios, &mut archivos)?;
    let mut publicados: Vec<(PathBuf, Option<PathBuf>)> = Vec::with_capacity(archivos.len());
    for archivo in archivos {
        match publicar_archivo(&archivo, tmp, destino, bak) {
            Ok(par) => publicados.push(par),
            Err(e) => {
                return Err(Error::Invalid(format!(
                    "{e}; {}",
                    detalle_reversion(&publicados, bak)
                )));
            }
        }
    }
    // Directories with no files in the archive (empty folders) are recreated too.
    for d in directorios {
        let rel = d.strip_prefix(tmp).map_err(|e| {
            Error::Invalid(
                RUTA_FUERA_TEMPORAL.texto_con(Idioma::de_entorno(), &[("e", &e.to_string())]),
            )
        })?;
        let objetivo = destino.join(rel);
        if let Err(e) = fs::create_dir_all(&objetivo) {
            return Err(Error::io(
                IMPORTACION_CANCELADA.texto_con(
                    Idioma::de_entorno(),
                    &[
                        ("destino", &objetivo.to_string_lossy()),
                        ("e", &e.to_string()),
                        ("reversion", &detalle_reversion(&publicados, bak)),
                    ],
                ),
                e,
            ));
        }
    }
    Ok(publicados.into_iter().map(|(p, _)| p).collect())
}

/// Move one extracted file into `destino`, backing up an existing target first.
fn publicar_archivo(
    archivo: &Path,
    tmp: &Path,
    destino: &Path,
    bak: &Path,
) -> Result<(PathBuf, Option<PathBuf>)> {
    let rel = archivo.strip_prefix(tmp).map_err(|e| {
        Error::Invalid(
            RUTA_FUERA_TEMPORAL.texto_con(Idioma::de_entorno(), &[("e", &e.to_string())]),
        )
    })?;
    let objetivo = destino.join(rel);
    if let Some(parent) = objetivo.parent() {
        fs::create_dir_all(parent).map_err(|e| {
            Error::io(
                NO_CREAR.texto_con(Idioma::de_entorno(), &[("ruta", &parent.to_string_lossy())]),
                e,
            )
        })?;
    }
    let respaldo = if objetivo.exists() {
        let b = bak.join(rel);
        if let Some(parent) = b.parent() {
            fs::create_dir_all(parent).map_err(|e| {
                Error::io(
                    NO_CREAR
                        .texto_con(Idioma::de_entorno(), &[("ruta", &parent.to_string_lossy())]),
                    e,
                )
            })?;
        }
        fs::rename(&objetivo, &b).map_err(|e| {
            Error::io(
                NO_RESPALDAR.texto_con(
                    Idioma::de_entorno(),
                    &[("ruta", &objetivo.to_string_lossy())],
                ),
                e,
            )
        })?;
        Some(b)
    } else {
        None
    };
    if let Err(e) = fs::rename(archivo, &objetivo) {
        return Err(error_de_publicacion(e, &objetivo, respaldo.as_deref()));
    }
    Ok((objetivo, respaldo))
}
