use super::*;
use crate::idioma::Idioma;
use serde_json::json;

/// GUI-06: `texto` was read by nobody and the click went to the window itself.
#[test]
fn should_refuse_texto_and_id_that_no_click_reads() {
    for args in [
        json!({ "ventana": "TDimensForm", "texto": "Sitio y datos meteo" }),
        json!({ "ventana": "TDimensForm", "id": 23 }),
    ] {
        let error = exigir_sin_texto_ni_id(Pedido {
            args: &args,
            idioma: Idioma::Es,
        })
        .expect_err("ignored argument");

        assert!(error.to_string().contains("clic_texto"), "{error}");
    }
}

/// 4.1-7 through the action: two OK buttons in two windows are not guessed.
#[test]
fn should_refuse_a_click_by_a_title_that_two_controls_share() {
    use crate::adapters::gui::ventana_falsa::{Hijo, VentanaFalsa};
    let uno = VentanaFalsa::crear(
        "TFormaUno",
        "Uno",
        vec![Hijo::de("TButton", (5, 5, 80, 30)).con_texto("OK")],
    );
    let otra = VentanaFalsa::crear(
        "TFormaOtra",
        "Otra",
        vec![Hijo::de("TButton", (5, 5, 80, 30)).con_texto("OK")],
    );
    let args = json!({ "ventana": "OK", "pids": [std::process::id()] });

    let error = invocar(&args, Idioma::Es)
        .expect_err("ambiguous")
        .to_string();

    assert!(error.contains("hwnd"), "{error}");
    assert!(error.contains(&format!("0x{:X}", uno.hwnd(1))), "{error}");
    assert!(error.contains(&format!("0x{:X}", otra.hwnd(1))), "{error}");
}

/// 4.1-1: `activar` on a minimised window restores it, it does not report success on an icon.
#[test]
fn should_restore_a_minimised_window_on_activar() {
    use crate::adapters::gui::ventana_falsa::VentanaFalsa;
    let forma = VentanaFalsa::crear("TFormaIcono", "Icono", vec![]);
    forma.minimizar();
    let args = json!({
        "accion": "activar",
        "ventana": forma.referencia(0),
        "pids": [std::process::id()],
    });

    invocar(&args, Idioma::Es).expect("restores it");

    assert!(!ffi::estado_ventana(forma.hwnd(0)).minimizada);
}
