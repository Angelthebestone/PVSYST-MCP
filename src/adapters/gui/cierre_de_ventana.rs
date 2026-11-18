//! Closing a window the way a user does: its own close/cancel button first,
//! `WM_CLOSE` only when the form has none, and the `closed` / `descartado` the
//! answer reports. A message box with no harmless button is a question: it is
//! left open and refused.

use super::acciones::clic_por_mensajes;
use super::ffi::{self, Hwnd, WM_CLOSE};
use super::rotulos::dialogos::{
    BOTONES_CANCELAR, BOTONES_CIERRE, BOTONES_EXCEPCION, BOTONES_MENSAJE,
};
use super::rotulos::Rotulo;
use super::ventanas::{class_of, text_of, visible};
use crate::error::{Error, Result};
use serde_json::{json, Value};

/// Class of PVsyst's message boxes.
pub(crate) const CLASE_MENSAJE: &str = "TMessageForm";
/// Class of the madExcept crash dialog.
pub(crate) const CLASE_EXCEPCION: &str = "madExceptWndClass";

fn botones_de_cierre(h: Hwnd) -> &'static [Rotulo<'static>] {
    match class_of(h).as_str() {
        CLASE_MENSAJE => BOTONES_MENSAJE,
        CLASE_EXCEPCION => BOTONES_EXCEPCION,
        _ => BOTONES_CIERRE,
    }
}

/// Buttons that close `h`: with `cancelar`, Cancelar/No come first.
fn botones_para_cerrar(h: Hwnd, cancelar: bool) -> Vec<Rotulo<'static>> {
    let base = botones_de_cierre(h);
    if cancelar {
        BOTONES_CANCELAR.iter().chain(base).copied().collect()
    } else {
        base.to_vec()
    }
}

/// Closes a VCL form the way a user does: its own close/cancel button first,
/// `WM_CLOSE` only when the form has none. `cancelar:true` tries Cancelar/No
/// before the default buttons. A message box with no harmless button (a Sí/No
/// question) is left open and reported.
pub(crate) fn cerrar_ventana(h: Hwnd, args: &Value) -> Result<()> {
    let botones = botones_para_cerrar(h, crate::args::flag(args, &["cancelar"]));
    let boton = ffi::enum_children(h)
        .into_iter()
        .filter(|c| visible(*c))
        .find(|c| botones.iter().any(|b| b.coincide(&text_of(*c))));
    match boton {
        Some(b) => clic_por_mensajes(b)?,
        None if class_of(h) == CLASE_MENSAJE => {
            return Err(Error::Gui(format!(
                "Message 0x{h:X} is a question: read it with gui_observar que:formulario or controles and answer with click on the chosen button"
            )))
        }
        None => {
            ffi::post_message(h, ffi::Mensaje::simple(WM_CLOSE));
        }
    }
    Ok(())
}

/// Marks the answer of a close: `closed`, and `descartado` when the window had
/// unsaved changes and the question was answered yes.
pub(super) fn anotar_cierre(window: &mut Value, descartado: bool) {
    if let Some(map) = window.as_object_mut() {
        map.insert("closed".into(), json!(true));
        map.insert("descartado".into(), json!(descartado));
    }
}
