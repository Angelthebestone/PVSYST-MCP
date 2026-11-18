//! Reversal of a failed import: published files are removed and their backups
//! put back, and the error names the backup when the reversal itself fails.

use crate::error::Error;
use crate::idioma::{Idioma, Texto};
use std::fs;
use std::path::{Path, PathBuf};

const NO_DEVOLVER_RESPALDO: Texto = Texto {
    es: "Importación cancelada en {destino}: no se pudo devolver el respaldo {respaldo} a su sitio ({e2}); recupéralo a mano desde esa ruta",
    en: "Import cancelled at {destino}: the backup {respaldo} could not be put back ({e2}); recover it by hand from that path",
};
const IMPORTACION_CANCELADA_SIMPLE: Texto = Texto {
    es: "Importación cancelada en {destino}",
    en: "Import cancelled at {destino}",
};
const COPIAS_EN: Texto = Texto {
    es: "copias de los archivos reemplazados en: {ruta}",
    en: "copies of the replaced files in: {ruta}",
};

/// Puts the backup of `objetivo` back after a failed publication and builds
/// the error. If the backup cannot be moved back it stays where it is and the
/// message names it, so the user can recover the original by hand.
pub(crate) fn error_de_publicacion(
    fallo: std::io::Error,
    objetivo: &Path,
    respaldo: Option<&Path>,
) -> Error {
    let devolucion = respaldo.map(|b| (b, fs::rename(b, objetivo)));
    match devolucion {
        Some((b, Err(e2))) => Error::io(
            NO_DEVOLVER_RESPALDO.texto_con(
                Idioma::de_entorno(),
                &[
                    ("destino", &objetivo.to_string_lossy()),
                    ("respaldo", &b.to_string_lossy()),
                    ("e2", &e2.to_string()),
                ],
            ),
            fallo,
        ),
        _ => Error::io(
            IMPORTACION_CANCELADA_SIMPLE.texto_con(
                Idioma::de_entorno(),
                &[("destino", &objetivo.to_string_lossy())],
            ),
            fallo,
        ),
    }
}

/// Undo published entries (newest first): remove the new file and put its
/// backup back. Returns a description and any failed reversions.
fn revertir_publicados(publicados: &[(PathBuf, Option<PathBuf>)]) -> (String, Vec<String>) {
    let mut fallos = Vec::new();
    for (objetivo, respaldo) in publicados.iter().rev() {
        if let Err(e) = fs::remove_file(objetivo) {
            fallos.push(format!("{}: {e}", objetivo.display()));
        }
        if let Some(b) = respaldo {
            if let Err(e) = fs::rename(b, objetivo) {
                fallos.push(format!("{}: {e}", b.display()));
            }
        }
    }
    if fallos.is_empty() {
        (
            Texto {
                es: "las entradas ya publicadas se restauraron",
                en: "the already published entries were restored",
            }
            .en_idioma(Idioma::de_entorno())
            .to_string(),
            fallos,
        )
    } else {
        (
            Texto {
                es: "la reversión falló",
                en: "the reversal failed",
            }
            .en_idioma(Idioma::de_entorno())
            .to_string(),
            fallos,
        )
    }
}

/// Description of the reversal for an error message; points at `bak` when the
/// reversal was partial, since it still holds replaced files.
pub(super) fn detalle_reversion(publicados: &[(PathBuf, Option<PathBuf>)], bak: &Path) -> String {
    let (reversion, fallos) = revertir_publicados(publicados);
    if fallos.is_empty() {
        reversion
    } else {
        format!(
            "{reversion}; {}",
            COPIAS_EN.texto_con(Idioma::de_entorno(), &[("ruta", &bak.to_string_lossy())],)
        )
    }
}
