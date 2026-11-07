use super::{
    aviso_de_foco, exigir_foco_en_la_ventana, exigir_objetivo_habilitado, resolver_foco, Destino,
    Objetivo,
};
use crate::error::Error;

#[test]
fn should_refuse_to_send_the_keys_when_the_control_did_not_take_the_focus_on_the_user_desktop() {
    let resultado = resolver_foco(false, Destino::Usuario, 0x10);

    assert!(matches!(resultado, Err(Error::Gui(m)) if m.contains("0x10")));
}

#[test]
fn should_send_the_keys_anyway_when_the_control_did_not_take_the_focus_on_the_isolated_desktop() {
    assert!(matches!(
        resolver_foco(false, Destino::Aislado, 0x10),
        Ok(false)
    ));
}

#[test]
fn should_refuse_the_keys_when_a_modal_disables_the_target() {
    let error = exigir_objetivo_habilitado(false, "THomeForm")
        .expect_err("a disabled target must be refused")
        .to_string();

    assert!(error.contains("THomeForm"), "{error}");
    assert!(error.contains("modal"), "{error}");
    assert!(error.contains("accion:cerrar"), "{error}");
}

#[test]
fn should_send_the_keys_when_the_target_is_enabled() {
    assert!(exigir_objetivo_habilitado(true, "THomeForm").is_ok());
}

#[test]
fn should_refuse_the_keys_when_the_focus_is_in_another_window() {
    let error = exigir_foco_en_la_ventana(0x20, 0x10, Some(0x30))
        .expect_err("a focus outside the target must be refused")
        .to_string();

    assert!(error.contains("0x20"), "{error}");
    assert!(error.contains("0x10"), "{error}");
    assert!(error.contains("accion:activar"), "{error}");
}

#[test]
fn should_accept_the_keys_when_the_focus_is_inside_the_target() {
    assert!(exigir_foco_en_la_ventana(0x11, 0x10, Some(0x10)).is_ok());
}

#[test]
fn should_accept_the_keys_when_no_control_reports_the_focus() {
    assert!(exigir_foco_en_la_ventana(0, 0x10, None).is_ok());
}

#[test]
fn should_explain_that_the_keys_reach_the_focused_control_when_the_target_is_the_window() {
    let aviso = aviso_de_foco(false, Objetivo::Ventana).expect("a warning");

    assert!(aviso.contains("focus"), "{aviso}");
}

#[test]
fn should_not_warn_when_a_control_took_the_focus() {
    assert_eq!(aviso_de_foco(true, Objetivo::Control), None);
}
