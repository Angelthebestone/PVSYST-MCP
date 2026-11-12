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
