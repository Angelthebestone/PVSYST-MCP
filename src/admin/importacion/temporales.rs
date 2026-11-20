//! The staging area beside the destination: the temp paths the import uses and
//! the extraction that fills them.

use super::NO_CREAR;
use crate::admin::archive::run_tar;
use crate::error::{Error, Result};
use crate::idioma::Idioma;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

/// Sibling temp paths for `destino`: extraction staging and replaced-file backup.
pub(super) fn rutas_temporales(destino: &Path) -> (PathBuf, PathBuf) {
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
pub(super) fn extraer(zip: &Path, tmp: &Path) -> Result<()> {
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
