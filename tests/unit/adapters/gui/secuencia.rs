use super::*;

#[test]
fn should_reject_an_empty_step_list() {
    let args = json!({"pasos": []});
    assert!(pasos_de(&args).is_err());
}

#[test]
fn should_reject_a_nested_sequence_before_running_anything() {
    let args = json!({"pasos": [
        {"accion": "esperar", "esperar_segundos": 0},
        {"accion": "secuencia", "pasos": []},
    ]});
    let err = pasos_de(&args).unwrap_err().to_string();
    assert!(err.contains("'pasos'[1]"), "{err}");
}

#[test]
fn should_reject_a_step_without_accion_naming_its_index() {
    let args = json!({"pasos": [{"accion": "esperar"}, {"ventana": "0x1"}]});
    let err = pasos_de(&args).unwrap_err().to_string();
    assert!(err.contains("'pasos'[1] necesita 'accion'"), "{err}");
}

#[test]
fn should_accept_a_list_of_named_actions_in_order() {
    let args = json!({"pasos": [{"accion": "esperar"}, {"accion": "cerrar"}]});
    let pasos = pasos_de(&args).expect("valid sequence");
    assert_eq!(pasos.len(), 2);
    assert_eq!(pasos[1]["accion"], "cerrar");
}
