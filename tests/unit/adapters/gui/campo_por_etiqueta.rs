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
