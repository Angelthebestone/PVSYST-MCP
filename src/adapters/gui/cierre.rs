//! What a close leaves behind. Closing an editor can make its parent selector
//! reappear as a modal, so the answer is built only after re-reading the forms.

use super::acciones::CLASE_MENSAJE;
use super::conduccion::{esperar_cierre, formularios};
use super::ffi::{self, Hwnd};
use super::modal;
use super::proceso::pids_pvsyst;
use super::ventanas::{class_of, pid_of, text_of, visible};
use crate::error::{Error, Result};
use serde_json::{json, Value};
use std::time::Duration;

/// Time PVsyst takes to show the parent form again after an editor closes.
pub(crate) const ESPERA_REAPARICION: Duration = Duration::from_millis(400);
/// Longest wait for a form to disappear after asking it to close.
const ESPERA_CIERRE: Duration = Duration::from_secs(2);
/// The home screen is always behind: it is not a form the close reopened.
const INICIO: &str = "THomeForm";

/// A visible top-level window of the process, as seen before and after a close.
pub(crate) struct VentanaViva {
    h: Hwnd,
    clase: String,
    titulo: String,
}

/// The visible forms of `pid` right now.
pub(crate) fn instantanea(pid: u32) -> Vec<VentanaViva> {
    let mut vivas: Vec<VentanaViva> = formularios(pid)
        .into_iter()
        .map(|v| VentanaViva {
            h: v,
            clase: class_of(v),
            titulo: text_of(v),
        })
        .collect();
    // A modal embedded as a child window is not in the top-level list, and it is
    // exactly the form that can block the next call. It can belong to another
    // process of the same executable, so every PVsyst process is searched.
    let mut pids = pids_pvsyst();
    if !pids.contains(&pid) {
        pids.push(pid);
    }
    if let Some(h) = modal::modal_de(&pids) {
        if !vivas.iter().any(|v| v.h == h) {
            vivas.push(VentanaViva {
                h,
                clase: class_of(h),
                titulo: text_of(h),
            });
        }
    }
    vivas
}

/// Forms the close made visible again: a bare `closed:true` would hide them.
pub(crate) fn reabiertas(
    antes: &[VentanaViva],
    despues: &[VentanaViva],
    cerrada: Hwnd,
) -> Vec<Value> {
    despues
        .iter()
        .filter(|v| v.h != cerrada && v.clase != INICIO && !antes.iter().any(|a| a.h == v.h))
        .map(|v| {
            json!({
                "hwnd": format!("0x{:X}", v.h),
                "class": v.clase,
                "title": v.titulo,
            })
        })
        .collect()
}

/// Names the reopened form and the `cerrar` that closes it, so the caller does
/// not have to discover the class with another observation.
pub(crate) fn aviso(reabiertas: &[Value]) -> Option<String> {
    let primera = reabiertas.first()?;
    let clase = primera["class"].as_str().unwrap_or_default();
    let hwnd = primera["hwnd"].as_str().unwrap_or_default();
    let resto = reabiertas.len().saturating_sub(1);
    let extra = if resto == 0 {
        String::new()
    } else {
        format!(" (and {resto} more)")
    };
    Some(format!(
        "Closing reopened {clase}{extra}: close it too with gui_actuar accion:cerrar ventana:{hwnd}"
    ))
}

/// A close that left the window open (it asked for a confirmation, or a modal
/// intercepted it) is a failure the caller must see.
pub(super) fn exigir_cerrada(h: Hwnd) -> Result<()> {
    if esperar_cierre(h, ESPERA_CIERRE) {
        return Ok(());
    }
    let confirmacion = modal::modal_de(&[pid_of(h)]).filter(|m| class_of(*m) == CLASE_MENSAJE);
    Err(Error::Gui(sigue_abierta(
        &class_of(h),
        confirmacion.map(|m| (m, botones_de(m))),
    )))
}

/// Visible buttons of `h` as `(hwnd, caption)`.
fn botones_de(h: Hwnd) -> Vec<(Hwnd, String)> {
    ffi::enum_children(h)
        .into_iter()
        .filter(|c| visible(*c))
        .map(|c| (c, class_of(c), text_of(c)))
        .filter(|(_, clase, _)| clase.contains("Btn") || clase.contains("Button"))
        .map(|(c, _, texto)| (c, texto))
        .collect()
}

/// Why a close failed: it opened a confirmation (named, with the buttons that
/// answer it) or it is unknown (GUI-14).
fn sigue_abierta(clase: &str, confirmacion: Option<(Hwnd, Vec<(Hwnd, String)>)>) -> String {
    let Some((mensaje, botones)) = confirmacion else {
        return format!(
            "{clase} is still open after asking it to close: it may have opened a confirmation (gui_observar que:ventanas)"
        );
    };
    let botones: Vec<String> = botones
        .iter()
        .map(|(b, texto)| format!("'{texto}' (0x{b:X})"))
        .collect();
    format!(
        "{clase} is still open: it asked for a confirmation in {CLASE_MENSAJE} 0x{mensaje:X}. Answer it with gui_actuar accion:click ventana:<hwnd> on one of its buttons: {}",
        botones.join(", ")
    )
}

#[cfg(test)]
#[path = "../../../tests/unit/adapters/gui/cierre.rs"]
mod tests;
