use super::*;
use crate::adapters::gui::ventana_falsa::{Hijo, VentanaFalsa};

/// BUG-15 of the exhaustive QA: with a modal open on THomeForm the invocation
/// hung for 10 s and answered about another window. It now names the modal first.
#[test]
fn should_refuse_to_invoke_on_a_form_that_a_modal_disables() {
    let principal = VentanaFalsa::crear(
        "THomeForm",
        "Inicio",
        vec![Hijo::de("TDatabaseForm", (50, 50, 300, 200))],
    );
    principal.deshabilitar();
    let args = json!({
        "ventana": principal.referencia(0),
        "nombre": "Herramientas",
        "rol": "button",
        "operacion": "invocar",
        "pids": [std::process::id()],
    });

    let error = uia_invocar(&args).expect_err("a disabled form").to_string();

    assert!(
        error.contains("modal") && error.contains("TDatabaseForm"),
        "{error}"
    );
}
