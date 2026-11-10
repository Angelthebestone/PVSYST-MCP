//! Transactional workspace import from a zip.
//!
//! The archive is extracted into a sibling staging directory and only then
//! published into the destination; files replaced during publication are
//! backed up and restored if any later step fails.

use crate::admin::archive::{run_tar, validar_entradas_zip};
use crate::args::{flag, req_str, workspace_of};
use crate::config::Paths;
use crate::error::{Error, Result};
use crate::idioma::{Idioma, Texto};
use crate::workspace::{resolver_ruta, Alcance};
use serde_json::{json, Value};
use std::fs;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

const ZIP_NO_ENCONTRADO: Texto = Texto {
    es: "Zip no encontrado: {zip} (resuelto contra el workspace {workspace}: pasa una ruta absoluta, o un 'zip' relativo a ese workspace)",
    en: "Zip not found: {zip} (resolved against the workspace {workspace}: pass an absolute path, or a 'zip' relative to that workspace)",
};
const DESTINO_NO_VACIO: Texto = Texto {
    es: "La carpeta destino no está vacía: {destino} (pasa sobrescribir:true)",
    en: "The destination folder is not empty: {destino} (pass sobrescribir:true)",
};
const NO_LEER_DESTINO: Texto = Texto {
    es: "No se pudo leer el destino {destino}",
    en: "Could not read the destination {destino}",
};
const RUTA_FUERA_TEMPORAL: Texto = Texto {
    es: "Ruta extraída fuera del temporal: {e}",
    en: "Extracted path outside the temporary folder: {e}",
};
const IMPORTACION_CANCELADA: Texto = Texto {
    es: "Importación cancelada en {destino}: {e}; {reversion}",
    en: "Import cancelled at {destino}: {e}; {reversion}",
};
const NO_CREAR: Texto = Texto {
    es: "No se pudo crear {ruta}",
    en: "Could not create {ruta}",
};
const NO_RESPALDAR: Texto = Texto {
    es: "No se pudo respaldar {ruta}",
    en: "Could not back up {ruta}",
};
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

/// Refuses a destination that holds files unless the caller asked to overwrite it.
fn exigir_destino_libre(paths: &Paths, destino: &Path, args: &Value) -> Result<()> {
    let no_vacia = match destino.read_dir() {
        Ok(mut d) => d.next().is_some(),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => false,
        Err(e) => {
            return Err(Error::io(
                NO_LEER_DESTINO.texto_con(paths.idioma, &[("destino", &destino.to_string_lossy())]),
                e,
            ))
        }
    };
    if no_vacia && !flag(args, &["sobrescribir"]) {
        return Err(Error::Invalid(DESTINO_NO_VACIO.texto_con(
            paths.idioma,
            &[("destino", &destino.to_string_lossy())],
        )));
    }
    Ok(())
}

/// Import a workspace from a zip into a destination folder.
pub(crate) fn importar_workspace(paths: &Paths, args: &Value) -> Result<Value> {
    let workspace = workspace_of(paths, args);
    let zip_str = req_str(args, &["zip"], "archive path")?;
    let zip = resolver_ruta(&workspace, zip_str, Alcance::de_args(args))?;
    if !zip.exists() {
        // A relative `zip` anchors to the call's workspace, as `destino` does: name it,
        // so a caller that exported into another workspace knows where to look.
        return Err(Error::NotFound(ZIP_NO_ENCONTRADO.texto_con(
            paths.idioma,
            &[
                ("zip", &zip.to_string_lossy()),
                ("workspace", &workspace.to_string_lossy()),
            ],
        )));
    }
    let dest_str = req_str(args, &["destino"], "folder to extract into")?;
    let destino = resolver_ruta(&workspace, dest_str, Alcance::de_args(args))?;
    exigir_destino_libre(paths, &destino, args)?;
    validar_entradas_zip(&zip)?;
    fs::create_dir_all(&destino).map_err(|e| {
        Error::io(
            NO_CREAR.texto_con(paths.idioma, &[("ruta", &destino.to_string_lossy())]),
            e,
        )
    })?;

    let (tmp, bak) = rutas_temporales(&destino);
    extraer(&zip, &tmp)?;
    let publicados = match publicar_extraccion(&tmp, &destino, &bak) {
        Ok(p) => p,
        Err(e) => {
            let _ = fs::remove_dir_all(&tmp);
            // Only an empty backup dir is removed; contents mean a failed
            // reversal whose copies must be kept.
            let _ = fs::remove_dir(&bak);
            return Err(e);
        }
    };
    let _ = fs::remove_dir_all(&tmp);
    let _ = fs::remove_dir_all(&bak);
    Ok(json!({
        "zip": zip.to_string_lossy(),
        "extracted_to": destino.to_string_lossy(),
        "archivos": publicados.len(),
    }))
}

/// Sibling temp paths for `destino`: extraction staging and replaced-file backup.
fn rutas_temporales(destino: &Path) -> (PathBuf, PathBuf) {
    let sello = format!(
        "{}_{}",
        std::process::id(),
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_or(0, |d| d.as_nanos())
    );
    (
        con_sufijo(destino, &format!(".mcp_import_tmp_{sello}")),
        con_sufijo(destino, &format!(".mcp_import_bak_{sello}")),
    )
}

/// Extract `zip` into the (new) `tmp`; removes any partial output on failure.
fn extraer(zip: &Path, tmp: &Path) -> Result<()> {
    fs::create_dir_all(tmp).map_err(|e| {
        Error::io(
            NO_CREAR.texto_con(Idioma::de_entorno(), &[("ruta", &tmp.to_string_lossy())]),
            e,
        )
    })?;
    if let Err(e) = run_tar(&[
        "-x".as_ref(),
        "-f".as_ref(),
        zip.as_os_str(),
        "-C".as_ref(),
        tmp.as_os_str(),
    ]) {
        let _ = fs::remove_dir_all(tmp);
        return Err(e);
    }
    Ok(())
}

/// `destino` with `sufijo` appended to its name; temp siblings share the volume.
fn con_sufijo(destino: &Path, sufijo: &str) -> PathBuf {
    let mut os = destino.as_os_str().to_owned();
    os.push(sufijo);
    PathBuf::from(os)
}

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
fn publicar_extraccion(tmp: &Path, destino: &Path, bak: &Path) -> Result<Vec<PathBuf>> {
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
fn detalle_reversion(publicados: &[(PathBuf, Option<PathBuf>)], bak: &Path) -> String {
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
