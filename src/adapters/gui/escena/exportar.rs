//! Saves the scene with PVsyst's own «Exportar escena (.SHD)»: PVsyst writes
//! the file, the MCP only names it and reads it back afterwards.
//!
//! PVsyst always writes a fresh temporary name. A read deletes that file when it
//! is done; a write moves it over the requested name. So a scene the MCP already
//! exported never makes the save dialog ask to overwrite, and a read never leaves
//! a file that blocks the next call with the same name.

use super::pasos::{abrir, con_descarte, descartar, hijo_n, ventana};
use crate::adapters::gui::conduccion::{esperar_cierre, Formulario};
use crate::adapters::gui::ffi;
use crate::adapters::gui::sondeo::sondear_hasta;
use crate::config::Paths;
use crate::error::{Error, Result};
use std::path::{Path, PathBuf};
use std::time::Duration;

const DIALOGO: &str = "TSaveObjPVForm";
const ESPERA_GUARDADO: Duration = Duration::from_secs(15);
const ESPERA_ARCHIVO: Duration = Duration::from_secs(20);
const SONDEO_ARCHIVO: Duration = Duration::from_millis(300);
/// Marks the files the MCP asks PVsyst to write for its own use.
const PREFIJO_TEMPORAL: &str = "_mcp_";

/// Where PVsyst saves a scene named `nombre`.
pub(super) fn ruta_de(paths: &Paths, nombre: &str) -> PathBuf {
    paths
        .workspace
        .join("Shadings")
        .join(format!("{nombre}.SHD"))
}

/// A name PVsyst has not written yet: the save dialog would otherwise ask to
/// overwrite an export of the MCP itself. It is deterministic so a retry after an
/// interrupted call removes the leftover instead of accumulating files.
fn ruta_temporal(paths: &Paths, nombre: &str) -> PathBuf {
    ruta_de(paths, &format!("{PREFIJO_TEMPORAL}{nombre}"))
}

/// A free temporary name, with any leftover of an interrupted call removed.
fn preparar_temporal(paths: &Paths, nombre: &str) -> PathBuf {
    let ruta = ruta_temporal(paths, nombre);
    borrar(&ruta);
    ruta
}

/// Exports the scene under a temporary name and returns the file PVsyst wrote.
pub(super) fn guardar_temporal(paths: &Paths, nombre: &str) -> Result<PathBuf> {
    let ruta = preparar_temporal(paths, nombre);
    abrir("escena_3d.archivo.exportar_escena")?;
    let dialogo = Formulario::de(ventana(DIALOGO)?);
    con_descarte(&dialogo, || {
        // Description above, file name below.
        let campo_nombre = hijo_n(dialogo.h, "TEdit", 1)?;
        ffi::traer_al_frente(dialogo.h);
        ffi::teclear_en(campo_nombre, &nombre_de(&ruta))?;
        dialogo.pulsar("Guardar")
    })?;
    let cerrado = esperar_cierre(dialogo.h, ESPERA_GUARDADO);
    if !cerrado {
        descartar(&dialogo);
    }
    if !cerrado || !esperar_escrito(&ruta) {
        return Err(Error::Gui(format!(
            "PVsyst did not save {}: no other way is attempted",
            ruta.display()
        )));
    }
    Ok(ruta)
}

/// Moves the temporary export over the requested name, replacing a previous
/// one. The scene is only kept once PVsyst has written it completely.
pub(super) fn conservar(temporal: &Path, definitiva: &Path) -> Result<PathBuf> {
    std::fs::rename(temporal, definitiva).map_err(|e| {
        Error::io(
            format!("could not keep the scene as {}", definitiva.display()),
            e,
        )
    })?;
    Ok(definitiva.to_path_buf())
}

/// Deletes an export the caller does not want to keep.
pub(super) fn borrar(ruta: &Path) {
    if let Err(error) = std::fs::remove_file(ruta) {
        if error.kind() != std::io::ErrorKind::NotFound {
            tracing::warn!(archivo = %ruta.display(), %error, "could not delete the temporary export");
        }
    }
}

fn nombre_de(ruta: &Path) -> String {
    ruta.file_stem()
        .map(|s| s.to_string_lossy().to_string())
        .unwrap_or_default()
}

/// PVsyst closes its dialog before it has written a large scene: the file counts as
/// written once it exists and its size stops changing between two polls.
fn esperar_escrito(ruta: &Path) -> bool {
    let mut anterior = None;
    sondear_hasta(ESPERA_ARCHIVO, SONDEO_ARCHIVO, || {
        let largo = std::fs::metadata(ruta).ok()?.len();
        let estable = largo > 0 && anterior == Some(largo);
        anterior = Some(largo);
        estable.then_some(())
    })
    .is_some()
}

#[cfg(test)]
#[path = "../../../../tests/unit/adapters/gui/escena/exportar.rs"]
mod tests;
