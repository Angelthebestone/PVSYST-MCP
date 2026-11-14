//! Restoring the window before an action that needs it on screen.

use super::{restaurar_al_frente, sigue_minimizada};
use crate::adapters::gui::ffi;
use crate::adapters::gui::ventana_falsa::VentanaFalsa;
use crate::idioma::Idioma;

#[test]
fn should_restore_a_minimised_window_before_acting_on_it() {
    let forma = VentanaFalsa::crear("TFormaMinimizada", "Minimizada", vec![]);
    forma.minimizar();
    assert!(
        ffi::estado_ventana(forma.hwnd(0)).minimizada,
        "precondition"
    );

    restaurar_al_frente(forma.hwnd(0), Idioma::Es).expect("restores");

    assert!(!ffi::estado_ventana(forma.hwnd(0)).minimizada);
}

#[test]
fn should_leave_a_window_that_is_not_minimised_as_it_is() {
    let forma = VentanaFalsa::crear("TFormaNormal", "Normal", vec![]);

    restaurar_al_frente(forma.hwnd(0), Idioma::Es).expect("nothing to restore");

    assert!(ffi::estado_ventana(forma.hwnd(0)).visible);
}

#[test]
fn should_name_the_window_that_stays_minimised() {
    let forma = VentanaFalsa::crear("TFormaQueSigue", "Sigue", vec![]);

    let error = sigue_minimizada(forma.hwnd(0), Idioma::En).to_string();

    assert!(error.contains("TFormaQueSigue"), "{error}");
    assert!(error.contains("IsIconic"), "{error}");
    assert!(
        error.contains(&format!("{:X}", forma.hwnd(0))),
        "the hwnd names the window: {error}"
    );
}
