//! Parsing of the `campo` selector used by the GUI field actions.

use super::*;
use serde_json::json;

#[test]
fn should_default_to_an_edit_field_and_keep_the_coordinates() {
    let campo = campo_de(&json!({ "campo": { "x": 1044, "y": 73 } })).expect("valid field");

    assert_eq!(campo.clase, "TEdit");
    assert_eq!((campo.x, campo.y), (1044, 73));
}

#[test]
fn should_map_the_supported_vcl_classes() {
    for (entrada, esperado) in [
        ("edit", "TEdit"),
        ("combo", "TComboBox"),
        ("opcion", "TRadioButton"),
        ("check", "TCheckBox"),
    ] {
        let campo = campo_de(&json!({ "campo": { "clase": entrada, "x": 1, "y": 2 } }))
            .expect("valid class");
        assert_eq!(campo.clase, esperado);
    }
}

#[test]
fn should_reject_an_unsupported_control_class() {
    let err = campo_de(&json!({ "campo": { "clase": "TTreeView", "x": 1, "y": 2 } }))
        .expect_err("unsupported class");
    assert!(err.to_string().contains("Clase de campo no soportada"));
}

#[test]
fn should_require_both_coordinates() {
    let err = campo_de(&json!({ "campo": { "x": 1 } })).expect_err("missing y");
    assert!(err.to_string().contains("campo.y"));
}

#[test]
fn should_reject_a_missing_field_selector() {
    let err = campo_de(&json!({})).expect_err("missing campo");
    assert!(err.to_string().contains("campo"));
}
