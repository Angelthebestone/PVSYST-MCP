//! Saves the scene with PVsyst's own «Exportar escena (.SHD)»: PVsyst writes
//! the file, the MCP only names it and reads it back afterwards.

use super::pasos::{abrir, hijo_n, ventana};
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

/// Where PVsyst saves a scene named `nombre`.
pub(super) fn ruta_de(paths: &Paths, nombre: &str) -> PathBuf {
    paths
        .workspace
        .join("Shadings")
        .join(format!("{nombre}.SHD"))
}

/// The path for `nombre`, refusing one that exists: PVsyst would ask to overwrite it.
pub(super) fn exigir_libre(paths: &Paths, nombre: &str) -> Result<PathBuf> {
    let ruta = ruta_de(paths, nombre);
    if ruta.exists() {
        return Err(Error::Invalid(format!(
            "Ya existe {}: elige otro nombre de escena",
            ruta.display()
        )));
    }
    Ok(ruta)
}

/// Exports the scene as `nombre` and returns the file PVsyst wrote.
pub(super) fn guardar(paths: &Paths, nombre: &str) -> Result<PathBuf> {
    let ruta = exigir_libre(paths, nombre)?;
    abrir("escena_3d.archivo.exportar_escena")?;
    let dialogo = Formulario::de(ventana(DIALOGO)?);
    // Description above, file name below.
    let campo_nombre = hijo_n(dialogo.h, "TEdit", 1)?;
    ffi::traer_al_frente(dialogo.h);
    ffi::teclear_en(campo_nombre, nombre)?;
    dialogo.pulsar("Guardar")?;
    if !esperar_cierre(dialogo.h, ESPERA_GUARDADO) || !esperar_escrito(&ruta) {
        return Err(Error::Gui(format!(
            "PVsyst no guardó {}: no se intenta de otra forma",
            ruta.display()
        )));
    }
    Ok(ruta)
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
