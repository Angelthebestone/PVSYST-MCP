//! Transactional workspace import from a zip.
//!
//! The archive is extracted into a sibling staging directory and only then
//! published into the destination; files replaced during publication are
//! backed up and restored if any later step fails.
//!
//! Each step sits in its own file: the destination guard in `destino`, the
//! staging area in `temporales`, the move into the destination in
//! `publicacion` and the reversal of a failed import in `reversion`.

mod destino;
mod publicacion;
mod reversion;
mod temporales;

pub(crate) use reversion::error_de_publicacion;

use crate::admin::archive::validar_entradas_zip;
use crate::args::{req_str, workspace_of};
use crate::config::Paths;
use crate::error::{Error, Result};
use crate::idioma::Texto;
use crate::workspace::{resolver_ruta, Alcance};
use destino::exigir_destino_libre;
use publicacion::publicar_extraccion;
use serde_json::{json, Value};
use std::fs;
use temporales::{extraer, rutas_temporales};

const ZIP_NO_ENCONTRADO: Texto = Texto {
    es: "Zip no encontrado: {zip} (resuelto contra el workspace {workspace}: pasa una ruta absoluta, o un 'zip' relativo a ese workspace)",
    en: "Zip not found: {zip} (resolved against the workspace {workspace}: pass an absolute path, or a 'zip' relative to that workspace)",
};

/// Shared by the steps that create a folder, so its message has one owner.
pub(super) const NO_CREAR: Texto = Texto {
    es: "No se pudo crear {ruta}",
    en: "Could not create {ruta}",
};

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
