//! The listing side of the modal rule: the blocking modal must be in the answer
//! even when it is a child window the enumeration does not return.

use super::{agregar_modal, puede_bloquear};
use serde_json::json;

/// GUI-13: the error named the UAC overlay instead of the dialog on top.
#[test]
fn should_not_take_a_windows_overlay_for_the_dialog_that_blocks() {
    assert!(!puede_bloquear("UAC_InputIndicatorOverlayWnd"));
    for dialogo in ["TToolsForm", "TMessageForm", "madExceptWndClass", "#32770"] {
        assert!(puede_bloquear(dialogo), "{dialogo}");
    }
}

#[test]
fn should_list_the_blocking_modal_when_the_enumeration_missed_it() {
    let mut out = vec![json!({"hwnd": "0x10", "class": "THomeForm"})];

    agregar_modal(
        &mut out,
        Some(json!({"hwnd": "0x20", "class": "TDEMOForm"})),
    );

    assert_eq!(out.len(), 2);
    assert_eq!(out[1]["class"], "TDEMOForm");
    assert_eq!(out[1]["modal"], true);
    assert_eq!(out[1]["child"], true);
}

/// 4.1-2: every modal that makes a click null is named with its class and hwnd.
#[test]
fn should_name_the_class_and_hwnd_of_each_modal_that_blocks_a_form() {
    use crate::adapters::gui::ventana_falsa::{Hijo, VentanaFalsa};
    for clase in [
        "TDEMOForm",
        "TMessageForm",
        "TChoixObjetPVForm",
        "TDatabaseForm",
    ] {
        let principal = VentanaFalsa::crear(
            "TFormaBloqueada",
            "Bloqueada",
            vec![Hijo::de(clase, (50, 50, 300, 200))],
        );
        principal.deshabilitar();

        let error = super::exigir_habilitada(principal.hwnd(0), super::PARA_ACTUAR)
            .expect_err("a disabled form")
            .to_string();

        let esperado = format!("modal: {clase} 0x{:X}", principal.hwnd(1));
        assert!(error.contains(&esperado), "{esperado} not in {error}");
    }
}

#[test]
fn should_let_an_enabled_form_through() {
    use crate::adapters::gui::ventana_falsa::VentanaFalsa;
    let libre = VentanaFalsa::crear("TFormaLibre", "Libre", vec![]);

    assert!(super::exigir_habilitada(libre.hwnd(0), super::PARA_ACTUAR).is_ok());
}
