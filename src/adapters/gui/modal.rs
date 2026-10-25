//! Modal dialogs. A VCL modal disables every form below it, and Windows drops the
//! input sent to a disabled window without reporting anything, so an action on
//! such a window looks successful and does nothing. Actions refuse those targets.

use super::conduccion::formularios;
use super::ffi::{self, Hwnd, GA_ROOT};
use super::ventanas::{class_of, pid_of};
use crate::error::{Error, Result};

/// Hint completing "…antes de" for actions that click or type into a window.
pub(crate) const PARA_ACTUAR: &str = "actuar sobre ella";

/// Fails when the top-level window of `ventana` is disabled by a modal dialog.
/// `para` completes the hint: "…antes de {para}".
pub(crate) fn exigir_habilitada(ventana: Hwnd, para: &str) -> Result<()> {
    let raiz = ffi::get_ancestor(ventana, GA_ROOT);
    if !ffi::es_ventana(raiz) || ffi::estado_ventana(raiz).habilitada {
        return Ok(());
    }
    let bloqueante = formularios(pid_of(raiz))
        .into_iter()
        .find(|h| ffi::estado_ventana(*h).habilitada)
        .map(class_of);
    let sobre = bloqueante.map_or_else(String::new, |clase| format!(" ({clase})"));
    Err(Error::Gui(format!(
        "{} está deshabilitada: hay un diálogo modal abierto sobre ella{sobre}. Ciérralo (gui_actuar accion:cerrar) antes de {para}",
        class_of(raiz)
    )))
}
