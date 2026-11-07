//! What counts as the modal that blocks a form: an enabled child form, because
//! a VCL modal can be embedded and `EnumWindows` does not list it.

use super::es_modal_hijo;
use crate::adapters::gui::ffi::EstadoVentana;

const VISIBLE_Y_HABILITADA: EstadoVentana = EstadoVentana {
    visible: true,
    habilitada: true,
    minimizada: false,
};
const OCULTA: EstadoVentana = EstadoVentana {
    visible: false,
    ..VISIBLE_Y_HABILITADA
};
const DESHABILITADA: EstadoVentana = EstadoVentana {
    habilitada: false,
    ..VISIBLE_Y_HABILITADA
};

#[test]
fn should_treat_a_visible_enabled_child_form_as_the_modal() {
    assert!(es_modal_hijo("TDEMOForm", VISIBLE_Y_HABILITADA));
}

#[test]
fn should_ignore_a_hidden_child_form() {
    assert!(!es_modal_hijo("TDEMOForm", OCULTA));
}

#[test]
fn should_ignore_a_disabled_child_form() {
    assert!(!es_modal_hijo("TDEMOForm", DESHABILITADA));
}

#[test]
fn should_ignore_a_child_that_is_not_a_form() {
    assert!(!es_modal_hijo("TPanel", VISIBLE_Y_HABILITADA));
}

// The listing side of the same rule: the blocking modal must be in the answer
// even when it is a child window the enumeration does not return.
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
