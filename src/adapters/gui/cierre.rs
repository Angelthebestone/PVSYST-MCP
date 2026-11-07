//! What a close leaves behind. Closing an editor can make its parent selector
//! reappear as a modal, so the answer is built only after re-reading the forms.

use super::conduccion::formularios;
use super::ffi::Hwnd;
use super::modal;
use super::proceso::pids_pvsyst;
use super::ventanas::{class_of, text_of};
use serde_json::{json, Value};
use std::time::Duration;

/// Time PVsyst takes to show the parent form again after an editor closes.
pub(crate) const ESPERA_REAPARICION: Duration = Duration::from_millis(400);
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

#[cfg(test)]
#[path = "../../../tests/unit/adapters/gui/cierre.rs"]
mod tests;
