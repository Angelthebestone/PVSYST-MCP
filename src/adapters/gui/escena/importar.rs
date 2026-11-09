//! Imports a 3D mesh into the scene open in the editor with PVsyst's own «Importar una
//! escena 3D»: the menu entry, the standard file dialog and «Importar resultado». PVsyst
//! converts the mesh; the MCP names the file and reads back the scene PVsyst exports.

use super::pasos::{hijo_n, ventana};
use crate::adapters::gui::acciones::cerrar_ventana;
use crate::adapters::gui::conduccion::{esperar, esperar_cierre, formularios, Formulario};
use crate::adapters::gui::ffi::{self, Hwnd, Mensaje, WM_COMMAND};
use crate::adapters::gui::sondeo::sondear_hasta;
use crate::adapters::gui::ventanas::{class_of, pid_of};
use crate::error::{Error, Result};
use serde_json::{json, Value};
use std::path::{Path, PathBuf};
use std::time::Duration;

const DIALOGO_ARCHIVO: &str = "#32770";
const RESULTADO: &str = "TImportResultForm";
const MENSAJE: &str = "TMessageForm";
const SONDEO: Duration = Duration::from_millis(200);
/// «Archivo» is menu 0 of the editor, «Importar» its item 4, «Importar una escena 3D» its entry 0.
const ENTRADA_IMPORTAR: (u8, u8, u8) = (0, 4, 0);
/// Command id of the default button («Abrir») of a Windows file dialog.
const ID_ABRIR: usize = 1;
const EXTENSIONES: &[&str] = &["3ds", "dae", "pvc", "pvc2", "pvz2"];
/// PVsyst refuses to open a file under its install folder («Accès au fichier refusé»).
const CARPETAS_PROTEGIDAS: &[&str] = &["program files", "program files (x86)"];
const ESPERA_DIALOGO: Duration = Duration::from_secs(15);
/// Reading a large mesh is PVsyst's own work.
const ESPERA_RESULTADO: Duration = Duration::from_secs(120);
const ESPERA_CIERRE: Duration = Duration::from_secs(60);
/// Time «Cancelar» or the window close gets to take effect after a failed close.
const ESPERA_LIBERACION: Duration = Duration::from_secs(5);

/// The mesh file the request names, checked before PVsyst is touched. A relative
/// name is looked up in the workspace, like every other path the tools take.
pub(super) fn malla_valida(workspace: &Path, texto: &str) -> Result<PathBuf> {
    let ruta = workspace.join(texto);
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
    // PVsyst's dialog resolves a relative name against its own folder, not this process's.
    let absoluta = std::path::absolute(&ruta).map_err(|e| {
        Error::Invalid(format!(
            "No se pudo resolver la ruta {}: {e}",
            ruta.display()
        ))
    })?;
    exigir_fuera_de_carpeta_protegida(&absoluta)?;
    if !absoluta.is_file() {
        return Err(Error::NotFound(format!("No existe {}", ruta.display())));
    }
    // The real location: a junction or a symlink can lead into a protected folder.
    if let Ok(real) = std::fs::canonicalize(&absoluta) {
        exigir_fuera_de_carpeta_protegida(&real)?;
    }
    Ok(absoluta)
}

fn exigir_fuera_de_carpeta_protegida(ruta: &Path) -> Result<()> {
    if en_carpeta_protegida(ruta) {
        return Err(Error::Invalid(
            "PVsyst no abre archivos de Program Files: copia la malla a tu workspace".into(),
        ));
    }
    Ok(())
}

/// Whether a folder of `ruta` is one of PVsyst's protected ones, by whole components: the 8.3
/// alias of «Program Files» counts, a folder that merely contains the words does not.
pub(super) fn en_carpeta_protegida(ruta: &Path) -> bool {
    ruta.components().any(|c| {
        let nombre = c.as_os_str().to_string_lossy().to_ascii_lowercase();
        CARPETAS_PROTEGIDAS.contains(&nombre.as_str()) || es_alias_corto_de_program_files(&nombre)
    })
}

/// `PROGRA~1`, `PROGRA~2`: the short names Windows gives «Program Files» and «Program Files (x86)».
fn es_alias_corto_de_program_files(nombre: &str) -> bool {
    nombre
        .strip_prefix("progra~")
        .is_some_and(|n| !n.is_empty() && n.bytes().all(|b| b.is_ascii_digit()))
}

/// Opens «Importar una escena 3D», types `ruta` in the file dialog and waits for the
/// result form. Returns that form, still open. When any step fails, whatever the import left
/// open (file dialog, PVsyst's error box) is closed, so PVsyst is not left behind a modal.
pub(super) fn abrir_resultado(ruta: &Path) -> Result<Formulario> {
    let editor = ventana("TPerspectiveForm")?;
    let pid = pid_of(editor);
    let (menu, item, sub) = ENTRADA_IMPORTAR;
    let abierto = crate::adapters::gui::catalogo::menu(&json!({
        "ventana": format!("0x{editor:X}"),
        "menu": menu, "item": item, "subitem": sub,
    }))
    .and_then(|_| esperar_dialogo_de_archivo(pid))
    .and_then(|dialogo| escribir_ruta(dialogo.h, ruta))
    .and_then(|()| esperar_resultado(pid));
    if abierto.is_err() {
        cerrar_modales(pid);
    }
    abierto
}

/// Waits for the result form. PVsyst answers a mesh it cannot read with an error box instead,
/// which ends the wait at once rather than after `ESPERA_RESULTADO`.
fn esperar_resultado(pid: u32) -> Result<Formulario> {
    let abierto = sondear_hasta(ESPERA_RESULTADO, SONDEO, || {
        formularios(pid)
            .into_iter()
            .find(|h| [RESULTADO, MENSAJE].contains(&class_of(*h).as_str()))
    })
    .ok_or_else(|| {
        Error::Gui(format!(
            "PVsyst no abrió '{RESULTADO}' en {} s",
            ESPERA_RESULTADO.as_secs()
        ))
    })?;
    if class_of(abierto) == MENSAJE {
        return Err(Error::Gui(format!(
            "PVsyst mostró un mensaje en vez de '{RESULTADO}': no pudo leer la malla (¿corrupta, no soportada o en una carpeta protegida?)"
        )));
    }
    Ok(Formulario::de(abierto))
}

/// The forms an import can leave open, front-most first, out of `abiertas` (handle, class).
pub(super) fn modales_de_importacion(abiertas: &[(Hwnd, String)]) -> Vec<Hwnd> {
    abiertas
        .iter()
        .filter(|(_, clase)| [DIALOGO_ARCHIVO, MENSAJE, RESULTADO].contains(&clase.as_str()))
        .map(|(h, _)| *h)
        .collect()
}

/// Best effort: the failure of the import is what the caller reports.
fn cerrar_modales(pid: u32) {
    let abiertas: Vec<(Hwnd, String)> = formularios(pid)
        .into_iter()
        .map(|h| (h, class_of(h)))
        .collect();
    for h in modales_de_importacion(&abiertas) {
        if let Err(error) = cerrar_ventana(h, &json!({ "cancelar": true })) {
            tracing::warn!(ventana = h, %error, "no se pudo cerrar el diálogo de la importación");
        }
    }
}

/// The file dialog. A scene that already has objects makes PVsyst ask first «¿Desea despejar la
/// escena de sombreado actual?»: «No» keeps what is there and the import adds to it.
fn esperar_dialogo_de_archivo(pid: u32) -> Result<Formulario> {
    let primera = sondear_hasta(ESPERA_DIALOGO, SONDEO, || {
        formularios(pid)
            .into_iter()
            .find(|h| [DIALOGO_ARCHIVO, MENSAJE].contains(&class_of(*h).as_str()))
    })
    .ok_or_else(|| {
        Error::Gui(format!(
            "PVsyst no abrió '{DIALOGO_ARCHIVO}' en {} s",
            ESPERA_DIALOGO.as_secs()
        ))
    })?;
    if class_of(primera) != MENSAJE {
        return Ok(Formulario::de(primera));
    }
    Formulario::de(primera).pulsar("No")?;
    esperar(pid, DIALOGO_ARCHIVO, ESPERA_DIALOGO)
}

fn escribir_ruta(dialogo: Hwnd, ruta: &Path) -> Result<()> {
    let campo = ffi::enum_children(dialogo)
        .into_iter()
        .find(|c| class_of(*c) == "Edit")
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
/// When the click fails or the form stays open, the form is dropped with «Cancelar» (and the
/// window closed if even that fails), so PVsyst is not left behind a modal; the error says so.
pub(super) fn cerrar_resultado(resultado: &Formulario, boton: &str) -> Result<()> {
    let pulsado = resultado.pulsar(boton);
    if pulsado.is_ok() && esperar_cierre(resultado.h, ESPERA_CIERRE) {
        return Ok(());
    }
    let cerrado = liberar_resultado(resultado);
    let estado = if cerrado {
        "se canceló"
    } else {
        "sigue abierto"
    };
    Err(match pulsado {
        Err(error) => Error::Gui(format!(
            "No se pudo pulsar «{boton}» en '{RESULTADO}' ({error}); {estado}"
        )),
        Ok(()) => Error::Gui(format!(
            "PVsyst no cerró '{RESULTADO}' tras «{boton}»; {estado}"
        )),
    })
}

/// Best effort after a failed close: «Cancelar» first, then the window itself. Whether the form
/// is gone afterwards.
fn liberar_resultado(resultado: &Formulario) -> bool {
    if !ffi::es_ventana(resultado.h) {
        return true;
    }
    super::pasos::descartar(resultado);
    if esperar_cierre(resultado.h, ESPERA_LIBERACION) {
        return true;
    }
    if let Err(error) = cerrar_ventana(resultado.h, &json!({ "cancelar": true })) {
        tracing::warn!(%error, "no se pudo cerrar '{RESULTADO}' tras un fallo");
    }
    esperar_cierre(resultado.h, ESPERA_LIBERACION)
}

#[cfg(test)]
#[path = "../../../../tests/unit/adapters/gui/escena/importar.rs"]
mod tests;
