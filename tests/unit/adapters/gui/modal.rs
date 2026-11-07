//! The listing side of the modal rule: the blocking modal must be in the answer
//! even when it is a child window the enumeration does not return.

use super::agregar_modal;
use serde_json::json;

#[test]
fn should_list_the_blocking_modal_when_the_enumeration_missed_it() {
    let mut out = vec![json!({"hwnd": "0x10", "class": "THomeForm"})];

    agregar_modal(
        &mut out,
        Some(json!({"hwnd": "0x20", "class": "TDEMOForm"})),
    );

    assert_eq!(out.len(), 2);
    assert_eq!(out[1]["class"], "TDEMOForm");
    assert_eq!(out[1]["modal"], true);
    assert_eq!(out[1]["child"], true);
}
