use super::*;
use crate::progreso::SeguimientoTarea;
use crate::server::tools::{call_seguido, Solicitud};
use serde_json::json;

/// The `claves_validas` of the `vista` of a call that fails for a missing argument.
fn claves_de(tool: &str, args: &Value) -> Vec<String> {
    let paths = Paths::resolve();
    let seguimiento = SeguimientoTarea::inerte();
    let solicitud = Solicitud {
        paths: &paths,
        name: tool,
        args,
        seguimiento: &seguimiento,
    };
    let error = call_seguido(&solicitud).expect_err("a required key is missing");
    let vista = vista_de_error(&solicitud, &error).expect("known tool");
    vista["error"]["claves_validas"]
        .as_array()
        .into_iter()
        .flatten()
        .filter_map(|c| c.as_str().map(str::to_string))
        .collect()
}

#[test]
fn should_name_the_key_the_runtime_found_missing_when_the_static_list_does_not_have_it() {
    let args = json!({ "accion": "dimensionar", "proyecto": "Casa" });
    assert!(claves_de("proyecto", &args).contains(&"kwp".to_string()));
}

#[test]
fn should_name_the_key_of_a_required_value_the_action_checks_itself() {
    let args = json!({ "accion": "teclas" });
    assert!(claves_de("gui_actuar", &args).contains(&"teclas".to_string()));
}

#[test]
fn should_name_the_rows_a_batch_needs_when_the_action_is_prepare() {
    let args = json!({ "accion": "preparar", "proyecto": "Casa" });
    assert!(claves_de("simular", &args).contains(&"filas".to_string()));
}

#[test]
fn should_name_the_discriminator_when_the_call_has_no_action_and_no_default() {
    assert!(claves_de("proyecto", &json!({})).contains(&"accion".to_string()));
}
