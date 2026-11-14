use super::*;
use crate::adapters::gui::ventana_falsa::{Hijo, VentanaFalsa};

fn listar(pids: &[u32]) -> Vec<Value> {
    let r = ventanas(&json!({ "pids": pids, "visibles": true, "completo": true })).unwrap();
    r["windows"].as_array().unwrap().clone()
}

fn de<'a>(ventanas: &'a [Value], hwnd: &str) -> Option<&'a Value> {
    ventanas.iter().find(|w| w["hwnd"] == hwnd)
}

#[test]
fn should_say_which_window_is_foreground_iconic_and_the_modal_that_blocks_the_rest() {
    let principal = VentanaFalsa::crear(
        "TMainForm",
        "Proyecto",
        vec![Hijo::de("TModalForm", (50, 50, 300, 200))],
    );
    principal.deshabilitar();
    let libre = VentanaFalsa::crear("TFreeForm", "Libre", Vec::new());

    let lista = listar(&[std::process::id()]);

    let principal_w = de(&lista, &principal.referencia(0)).unwrap();
    let libre_w = de(&lista, &libre.referencia(0)).unwrap();
    for w in [principal_w, libre_w] {
        assert!(w["foreground"].is_boolean(), "{w}");
        assert_eq!(w["iconic"], false);
        assert_eq!(w["modal"], false);
    }
    assert_eq!(principal_w["enabled"], false);
    let bloqueante = de(&lista, &principal.referencia(1)).unwrap();
    assert_eq!(bloqueante["modal"], true);
}

/// `ocr` is read only when asked for: a plain `leer` costs no capture.
#[test]
fn should_read_the_window_text_and_leave_ocr_out_unless_it_is_asked_for() {
    let ventana = VentanaFalsa::crear("TFreeForm", "Libre", Vec::new());
    let paths = crate::config::Paths::para_test(std::env::temp_dir());
    let args =
        json!({ "que": "leer", "ventana": ventana.referencia(0), "pids": [std::process::id()] });

    let r = crate::adapters::gui::observar(&paths, &args).unwrap();

    assert_eq!(r["class"], "TFreeForm");
    assert!(r.get("ocr").is_none(), "{r}");
}
