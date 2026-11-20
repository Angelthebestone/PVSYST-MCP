//! A click on screen coordinates lands where the window is, not where it was: a
//! minimised window sits at (-32000, -32000), so a physical click or an OCR read
//! on it hits nothing and reports nothing. Actions that need the window on screen
//! restore it first and refuse, naming the window, when it stays minimised.

use super::ffi::{self, Hwnd, GA_ROOT, SW_RESTORE};
use super::sondeo::esperar_mientras;
use super::ventanas::class_of;
use crate::error::{Error, Result};
use crate::idioma::{Idioma, Texto};
use std::time::Duration;

/// Time a window gets to leave the minimised state after `SW_RESTORE`.
const ESPERA_RESTAURAR: Duration = Duration::from_millis(1500);
const SONDEO_RESTAURAR: Duration = Duration::from_millis(50);

/// The refusal for a window that is still minimised after restoring it.
fn sigue_minimizada(raiz: Hwnd, idioma: Idioma) -> Error {
    let sigue = Texto {
        es: "La ventana {clase} 0x{hwnd} sigue minimizada (IsIconic) tras restaurarla: un clic o una captura caería fuera. Restáurala a mano o con gui_actuar accion:activar y repite",
        en: "Window {clase} 0x{hwnd} is still minimised (IsIconic) after restoring it: a click or a capture would land outside. Restore it by hand or with gui_actuar accion:activar and retry",
    };
    Error::Gui(sigue.texto_con(
        idioma,
        &[("clase", &class_of(raiz)), ("hwnd", &format!("{raiz:X}"))],
    ))
}

/// Restores the top-level window of `h` when it is minimised and brings it to the
/// front. Fails, naming the window, when `IsIconic` is still true afterwards.
/// A window that is not minimised is only brought forward.
pub(crate) fn restaurar_al_frente(h: Hwnd, idioma: Idioma) -> Result<()> {
    let raiz = ffi::get_ancestor(h, GA_ROOT);
    if !ffi::es_ventana(raiz) {
        return Ok(());
    }
    if ffi::estado_ventana(raiz).minimizada {
        ffi::mostrar_ventana(raiz, SW_RESTORE);
        esperar_mientras(ESPERA_RESTAURAR, SONDEO_RESTAURAR, || {
            ffi::estado_ventana(raiz).minimizada
        });
    }
    ffi::traer_al_frente(raiz);
    if ffi::estado_ventana(raiz).minimizada {
        return Err(sigue_minimizada(raiz, idioma));
    }
    Ok(())
}

#[cfg(test)]
#[path = "../../../tests/unit/adapters/gui/primer_plano.rs"]
mod tests;
