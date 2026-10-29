//! Small steps the scene driver repeats: open a catalogued window, list the
//! controls of a class from top to bottom, type a value into a field.

use crate::adapters::gui::ffi::{self, Hwnd};
use crate::adapters::gui::ventanas::{class_of, hwnd_de, rect_of, visible};
use crate::error::{Error, Result};
use serde_json::{json, Value};
use std::time::Duration;

/// Lets the form react to a typed value before the next step.
const PAUSA: Duration = Duration::from_millis(400);

/// Opens a catalogue function (`gui_actuar accion:funcion`), opening its host when needed.
pub(super) fn abrir(funcion: &str) -> Result<Value> {
    // The menu commands need PVsyst in the foreground, and a window opened a moment ago may not be.
    if let Ok(editor) = ventana("TPerspectiveForm") {
        ffi::traer_al_frente(editor);
        std::thread::sleep(PAUSA);
    }
    crate::gui_catalogo::abrir(&json!({ "funcion": funcion }))
}

/// The open window of class `clase`.
pub(super) fn ventana(clase: &str) -> Result<Hwnd> {
    hwnd_de(&json!({ "ventana": clase }), "ventana")
}

/// Visible children of `padre` of class `clase`, top to bottom.
pub(super) fn hijos(padre: Hwnd, clase: &str) -> Vec<Hwnd> {
    let mut lista: Vec<Hwnd> = ffi::enum_children(padre)
        .into_iter()
        .filter(|c| visible(*c) && class_of(*c) == clase)
        .collect();
    lista.sort_by_key(|c| rect_of(*c).top);
    lista
}

/// Enabled edit fields of `padre`, top to bottom.
pub(super) fn campos(padre: Hwnd) -> Vec<Hwnd> {
    hijos(padre, "TEdit")
        .into_iter()
        .filter(|c| ffi::estado_ventana(*c).habilitada)
        .collect()
}

/// The `n`-th (from 0) child of class `clase`, or an error naming what is missing.
pub(super) fn hijo_n(padre: Hwnd, clase: &str, n: usize) -> Result<Hwnd> {
    hijos(padre, clase)
        .get(n)
        .copied()
        .ok_or_else(|| Error::Gui(format!("Falta el control {clase} nº {n} de la ventana")))
}

/// Types `texto` into `campo` as a user would: focus first, so the field commits
/// when Tab leaves it, and refuses to type when the focus did not move.
pub(super) fn escribir(formulario: Hwnd, campo: Hwnd, texto: &str) -> Result<()> {
    if !ffi::traer_y_focar(campo, PAUSA) {
        return Err(Error::Gui(format!(
            "No se pudo dar el foco al campo de la ventana 0x{formulario:X}: lo escrito iría a otro control"
        )));
    }
    ffi::teclear_en(campo, texto)?;
    ffi::pulsar_tabulador();
    std::thread::sleep(PAUSA);
    Ok(())
}
