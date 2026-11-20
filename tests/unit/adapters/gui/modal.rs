//! The listing side of the modal rule: the blocking modal must be in the answer
//! even when it is a child window the enumeration does not return.

use super::{agregar_modal, aviso_demo, puede_bloquear, AvisoDemo, CLASE_AVISO_DEMO};
use crate::adapters::gui::rotulos::dialogos::{CERRAR, SI};
use crate::adapters::gui::rotulos::IdiomaGui;
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

/// C.4 of the visual GUI test report: in trial mode PVsyst's DEMO notice blocks
/// every window and every action answered «… is disabled: a modal dialog is open
/// on top of it (modal: TDEMOForm 0x…)» until someone pressed «Cerrar». It only
/// informs, so its «Cerrar» may be pressed — and nothing else, because the other
/// three buy, activate a licence or open the help. A question or an error is not
/// answered at all, and a GUI whose «Cerrar» caption is not among the buttons
/// presses nothing.
#[test]
fn should_dismiss_the_demo_notice_with_its_close_button_and_no_other_dialog() {
    let medidos_en_espanol = ["Compra", "Activar", "Cerrar", "Ayuda"];
    assert_eq!(
        aviso_demo(CLASE_AVISO_DEMO, &medidos_en_espanol, Some(IdiomaGui::Es)),
        AvisoDemo::Cerrar { boton: CERRAR }
    );

    // Each language is recognised by the caption the server presses, not by the
    // words PVsyst drew next to it.
    for idioma in IdiomaGui::TODOS {
        let botones = ["Compra", "Activar", CERRAR.en_idioma(idioma), "Ayuda"];
        assert_eq!(
            aviso_demo(CLASE_AVISO_DEMO, &botones, Some(idioma)),
            AvisoDemo::Cerrar { boton: CERRAR },
            "{idioma:?}"
        );
    }

    // A notice with no known «Cerrar» — the GUI's language is not the one its
    // buttons are written in — is left alone.
    assert_eq!(
        aviso_demo(CLASE_AVISO_DEMO, &medidos_en_espanol, Some(IdiomaGui::En)),
        AvisoDemo::OtroDialogo
    );
    assert_eq!(
        aviso_demo(CLASE_AVISO_DEMO, &medidos_en_espanol, None),
        AvisoDemo::OtroDialogo
    );

    // Every other dialog is a decision for a person, even with a «Cerrar» and a
    // «Sí» on it.
    for dialogo in [
        "TMessageForm",
        "TDatabaseForm",
        "#32770",
        "TChoixObjetPVForm",
    ] {
        assert_eq!(
            aviso_demo(dialogo, &[CERRAR.es, SI.es], Some(IdiomaGui::Es)),
            AvisoDemo::OtroDialogo,
            "{dialogo}"
        );
    }
}

#[test]
fn should_let_an_enabled_form_through() {
    use crate::adapters::gui::ventana_falsa::VentanaFalsa;
    let libre = VentanaFalsa::crear("TFormaLibre", "Libre", vec![]);

    assert!(super::exigir_habilitada(libre.hwnd(0), super::PARA_ACTUAR).is_ok());
}
