use super::*;

#[test]
fn should_reject_a_nested_sequence_before_running_anything() {
    let args = json!({"pasos": [
        {"accion": "esperar", "esperar_segundos": 0},
        {"accion": "secuencia", "pasos": []},
    ]});
    let err = pasos_de(&args, crate::idioma::Idioma::Es)
        .unwrap_err()
        .to_string();
    assert!(err.contains("'pasos'[1]"), "{err}");
}
