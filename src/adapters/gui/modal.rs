//! Modal dialogs. A VCL modal disables every form below it, and Windows drops the
//! input sent to a disabled window without reporting anything, so an action on
//! such a window looks successful and does nothing. Actions refuse those targets.

use super::conduccion::formularios;
use super::ffi::{self, Hwnd, GA_ROOT};
use super::ventanas::{class_of, pid_of};
use crate::error::{Error, Result};

/// Hint completing "…before" for actions that click or type into a window.
pub(crate) const PARA_ACTUAR: &str = "acting on it";

/// A VCL modal can be a child window of the form it disables: `EnumWindows`
/// does not list it, so the blocker is looked for among the children first.
fn es_modal_hijo(clase: &str, estado: ffi::EstadoVentana) -> bool {
    estado.visible && estado.habilitada && clase.ends_with("Form")
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
        formularios(pid_of(raiz))
            .into_iter()
            .find(|h| *h != raiz && ffi::estado_ventana(*h).habilitada)
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

#[cfg(test)]
#[path = "../../../tests/unit/adapters/gui/modal.rs"]
mod tests;
