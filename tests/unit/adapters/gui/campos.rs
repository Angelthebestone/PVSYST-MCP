//! Parsing of the `campo` selector used by the GUI field actions.

use super::*;
use serde_json::json;

#[test]
fn should_require_both_coordinates_when_a_field_selector_is_given() {
    let err = campo_de(&json!({ "campo": { "x": 1 } })).expect_err("missing y");
    assert!(err.to_string().contains("campo.y"));
}
