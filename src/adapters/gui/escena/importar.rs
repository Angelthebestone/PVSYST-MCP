//! Imports a 3D mesh into the scene open in the editor with PVsyst's own «Importar una
//! escena 3D»: the menu entry, the standard file dialog and «Importar resultado». PVsyst
//! converts the mesh; the MCP names the file and reads back the scene PVsyst exports.

use super::pasos::{hijo_n, ventana};
use crate::adapters::gui::conduccion::{esperar, esperar_cierre, Formulario};
use crate::adapters::gui::ffi::{self, Hwnd, Mensaje, WM_COMMAND};
use crate::adapters::gui::ventanas::pid_of;
use crate::error::{Error, Result};
use serde_json::{json, Value};
use std::path::{Path, PathBuf};
use std::time::Duration;

const DIALOGO_ARCHIVO: &str = "#32770";
const RESULTADO: &str = "TImportResultForm";
/// «Archivo» is menu 0 of the editor, «Importar» its item 4, «Importar una escena 3D» its entry 0.
const ENTRADA_IMPORTAR: (u8, u8, u8) = (0, 4, 0);
/// Command id of the default button («Abrir») of a Windows file dialog.
const ID_ABRIR: usize = 1;
const EXTENSIONES: &[&str] = &["3ds", "dae", "pvc", "pvc2", "pvz2"];
/// PVsyst refuses to open a file under its install folder («Accès au fichier refusé»).
const CARPETA_PROTEGIDA: &str = "program files";
const ESPERA_DIALOGO: Duration = Duration::from_secs(15);
/// Reading a large mesh is PVsyst's own work.
const ESPERA_RESULTADO: Duration = Duration::from_secs(120);
const ESPERA_CIERRE: Duration = Duration::from_secs(60);

/// The mesh file the request names, checked before PVsyst is touched.
pub(super) fn malla_valida(texto: &str) -> Result<PathBuf> {
    let ruta = PathBuf::from(texto);
    let extension = ruta
        .extension()
        .and_then(|e| e.to_str())
        .map(str::to_ascii_lowercase)
        .unwrap_or_default();
    if !EXTENSIONES.contains(&extension.as_str()) {
        return Err(Error::Invalid(format!(
            "'archivo' debe ser una malla {}",
            EXTENSIONES.join(", ")
        )));
    }
    if texto.to_ascii_lowercase().contains(CARPETA_PROTEGIDA) {
        return Err(Error::Invalid(
            "PVsyst no abre archivos de Program Files: copia la malla a tu workspace".into(),
        ));
    }
    if !ruta.is_file() {
        return Err(Error::NotFound(format!("No existe {}", ruta.display())));
    }
    Ok(ruta)
}

/// Opens «Importar una escena 3D», types `ruta` in the file dialog and waits for the
/// result form. Returns that form, still open.
pub(super) fn abrir_resultado(ruta: &Path) -> Result<Formulario> {
    let editor = ventana("TPerspectiveForm")?;
    let (menu, item, sub) = ENTRADA_IMPORTAR;
    crate::gui_catalogo::menu(&json!({
        "ventana": format!("0x{editor:X}"),
        "menu": menu, "item": item, "subitem": sub,
    }))?;
    let dialogo = esperar(pid_of(editor), DIALOGO_ARCHIVO, ESPERA_DIALOGO)?;
    escribir_ruta(dialogo.h, ruta)?;
    esperar(pid_of(editor), RESULTADO, ESPERA_RESULTADO)
}

fn escribir_ruta(dialogo: Hwnd, ruta: &Path) -> Result<()> {
    let campo = ffi::enum_children(dialogo)
        .into_iter()
        .find(|c| crate::adapters::gui::ventanas::class_of(*c) == "Edit")
        .ok_or_else(|| Error::Gui("El diálogo de archivo no tiene campo de nombre".into()))?;
    ffi::teclear_en(campo, &ruta.display().to_string())?;
    if !ffi::post_message(dialogo, Mensaje::nuevo(WM_COMMAND, ID_ABRIR, 0)) {
        return Err(Error::Gui("No se pudo pulsar «Abrir» en el diálogo".into()));
    }
    Ok(())
}

/// Picks the unit of the mesh in «Importar resultado» (its first combo).
pub(super) fn elegir_unidades(resultado: &Formulario, unidades: &str) -> Result<()> {
    let lista = hijo_n(resultado.h, "TComboBox", 0)?;
    if ffi::elegir_en_combo(lista, unidades)? {
        return Ok(());
    }
    Err(Error::Invalid(format!(
        "PVsyst no ofrece las unidades '{unidades}' (p.ej. 'Metros', 'Pulgadas')"
    )))
}

/// The unit selected in the result form, to report what the import used.
pub(super) fn unidades_elegidas(resultado: &Formulario) -> Value {
    let combo = hijo_n(resultado.h, "TComboBox", 0).ok();
    json!(combo.map(ffi::texto_control))
}

/// Closes «Importar resultado» with `boton` («OK» applies the import, «Cancelar» drops it).
pub(super) fn cerrar_resultado(resultado: &Formulario, boton: &str) -> Result<()> {
    resultado.pulsar(boton)?;
    if esperar_cierre(resultado.h, ESPERA_CIERRE) {
        return Ok(());
    }
    Err(Error::Gui(format!(
        "PVsyst no cerró '{RESULTADO}' tras «{boton}»"
    )))
}

#[cfg(test)]
#[path = "../../../../tests/unit/adapters/gui/escena/importar.rs"]
mod tests;
