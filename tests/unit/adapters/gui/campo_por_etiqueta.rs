//! `gui_actuar accion:campo` against the answer `gui_observar que:formulario`
//! gives: a field PVsyst reports disabled must be refused, not written.

use super::*;
use serde_json::json;

/// One form field as the form answer describes it: label, class and whether
/// PVsyst reports it enabled.
fn ficha(habilitado: bool) -> Value {
    json!({
        "ventana": "0x100",
        "campos": [{
            "hwnd": "0x101",
            "clase": "TCheckBox",
            "etiqueta": "Activar pérdidas por sombreado",
            "habilitado": habilitado,
            "rect": {"left": 20, "top": 30, "right": 300, "bottom": 50},
        }],
    })
}

/// A state PVsyst could not report used to answer "desconocido" as the value
/// of the control (4.1 of the visual GUI test report).
#[test]
fn should_return_an_error_when_the_state_of_an_option_cannot_be_read() {
    let casilla = CampoEtiquetado {
        hwnd: 0,
        clase: "TCheckBox".into(),
        etiqueta: Some("Activar pérdidas por sombreado".into()),
        posicion: (0, 0),
    };

    let error = leer_valor(&casilla)
        .expect_err("an unreadable option is not a value")
        .to_string();

    assert!(error.contains("Activar pérdidas por sombreado"), "{error}");
    assert!(error.contains("TCheckBox"), "{error}");
    assert!(error.contains("gui_observar"), "{error}");
}

/// 4.1: the posted click was not awaited, so a check box that ignored it still
/// answered success with the old value as `despues`.
#[test]
fn should_fail_when_the_check_box_does_not_take_the_requested_value() {
    use crate::adapters::gui::ventana_falsa::{Hijo, VentanaFalsa};
    let forma = VentanaFalsa::crear(
        "TFormaCasilla",
        "Casilla",
        vec![Hijo::de("TCheckBox", (5, 5, 220, 30)).con_texto("Sombreado")],
    );
    let casilla = CampoEtiquetado {
        hwnd: forma.hwnd(1),
        clase: "TCheckBox".into(),
        etiqueta: Some("Sombreado".into()),
        posicion: (0, 0),
    };

    let error = esperar_marca(&casilla, true, Duration::from_millis(100))
        .expect_err("a click the control did not take is not a success")
        .to_string();

    assert!(error.contains("Sombreado"), "{error}");
    assert!(error.contains("'true'"), "{error}");
    assert!(error.contains("'false'"), "{error}");
    assert!(error.contains("gui_observar"), "{error}");
}

/// A disabled control swallows the text and the click without a trace, so
/// `accion:campo` answered `vista.estado:"ok"` with `antes == despues` (4.2 of
/// the visual GUI test report).
#[test]
fn should_refuse_a_field_that_pvsyst_reports_disabled() {
    let error = elegir_campo(&ficha(false), "Activar pérdidas", Desambiguador::default())
        .expect_err("a disabled field takes no value")
        .to_string();
    assert!(error.contains("Activar pérdidas"), "{error}");
    assert!(error.contains("disabled"), "{error}");
    assert!(error.contains("gui_observar que:formulario"), "{error}");

    let habilitado = elegir_campo(&ficha(true), "Activar pérdidas", Desambiguador::default())
        .expect("an enabled field takes the value");
    assert_eq!(habilitado.clase, "TCheckBox");
}
