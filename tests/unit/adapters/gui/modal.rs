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
