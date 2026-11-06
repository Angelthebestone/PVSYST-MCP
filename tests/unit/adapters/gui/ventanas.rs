//! Listing the windows of PVsyst: the modal that blocks everything must be
//! there even when it is a child window the enumeration does not return.

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

#[test]
fn should_not_duplicate_the_modal_when_it_is_already_listed() {
    let mut out = vec![json!({"hwnd": "0x20", "class": "TDEMOForm"})];

    agregar_modal(
        &mut out,
        Some(json!({"hwnd": "0x20", "class": "TDEMOForm"})),
    );

    assert_eq!(out.len(), 1);
}

#[test]
fn should_leave_the_list_alone_when_there_is_no_modal() {
    let mut out = vec![json!({"hwnd": "0x10", "class": "THomeForm"})];

    agregar_modal(&mut out, None);

    assert_eq!(out.len(), 1);
}
