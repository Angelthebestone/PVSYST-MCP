//! Modal dialogs. A VCL modal disables every form below it, and Windows drops the
//! input sent to a disabled window without reporting anything, so an action on
//! such a window looks successful and does nothing. Actions refuse those targets.

use super::acciones::CLASE_EXCEPCION;
use super::conduccion::formularios;
use super::ffi::{self, Hwnd, GA_ROOT};
use super::ventanas::{class_of, enum_windows_de, pid_of, visible};
use crate::error::{Error, Result};
use serde_json::{json, Value};

/// Class of the standard Windows dialogs (file open/save).
const CLASE_DIALOGO_WINDOWS: &str = "#32770";

/// Hint completing "…before" for actions that click or type into a window.
pub(crate) const PARA_ACTUAR: &str = "acting on it";

/// A VCL modal can be a child window of the form it disables: `EnumWindows`
/// does not list it, so the blocker is looked for among the children first.
fn es_modal_hijo(clase: &str, estado: ffi::EstadoVentana) -> bool {
    estado.visible && estado.habilitada && clase.ends_with("Form")
}

/// Whether a top-level window of this class can be a dialog that blocks a form:
/// a VCL form, the madExcept crash box or a standard Windows dialog. The overlays
/// Windows adds to the process (`UAC_InputIndicatorOverlayWnd`) are enabled and
/// visible but block nothing (GUI-13).
fn puede_bloquear(clase: &str) -> bool {
    clase.ends_with("Form") || clase == CLASE_EXCEPCION || clase == CLASE_DIALOGO_WINDOWS
}

/// The dialog blocking `raiz`: an enabled child form when the modal is embedded,
/// else the enabled top-level window above it.
pub(crate) fn bloqueante_de(raiz: Hwnd) -> Option<Hwnd> {
    if ffi::estado_ventana(raiz).habilitada {
        return None;
    }
    let hijo = ffi::enum_children(raiz)
        .into_iter()
        .find(|c| es_modal_hijo(&class_of(*c), ffi::estado_ventana(*c)));
    hijo.or_else(|| {
        formularios(pid_of(raiz)).into_iter().find(|h| {
            *h != raiz && ffi::estado_ventana(*h).habilitada && puede_bloquear(&class_of(*h))
        })
    })
}

/// Fails when the top-level window of `ventana` is disabled by a modal dialog.
/// `para` completes the hint: "…before {para}".
pub(crate) fn exigir_habilitada(ventana: Hwnd, para: &str) -> Result<()> {
    let raiz = ffi::get_ancestor(ventana, GA_ROOT);
    if !ffi::es_ventana(raiz) || ffi::estado_ventana(raiz).habilitada {
        return Ok(());
    }
    let sobre = bloqueante_de(raiz)
        .map(class_of)
        .map_or_else(String::new, |clase| format!(" ({clase})"));
    Err(Error::Gui(format!(
        "{} is disabled: a modal dialog is open on top of it{sobre}. Close it (gui_actuar accion:cerrar) before {para}",
        class_of(raiz)
    )))
}

/// The modal blocking the process, if any: a VCL modal can be a child window of
/// the form it disables, and `EnumWindows` would not list it.
pub(crate) fn modal_de(pids: &[u32]) -> Option<Hwnd> {
    pids.iter()
        .flat_map(|p| enum_windows_de(*p))
        .filter(|h| visible(*h))
        .find_map(bloqueante_de)
}

/// Adds the blocking modal when the enumeration did not list it, without
/// duplicating one that is already there.
pub(crate) fn agregar_modal(out: &mut Vec<Value>, modal: Option<Value>) {
    let Some(mut modal) = modal else {
        return;
    };
    let hwnd = modal.get("hwnd").cloned().unwrap_or(Value::Null);
    if out.iter().any(|w| w.get("hwnd") == Some(&hwnd)) {
        return;
    }
    if let Some(mapa) = modal.as_object_mut() {
        mapa.insert("modal".into(), json!(true));
        mapa.insert("child".into(), json!(true));
    }
    out.push(modal);
}

#[cfg(test)]
#[path = "../../../tests/unit/adapters/gui/modal.rs"]
mod tests;
