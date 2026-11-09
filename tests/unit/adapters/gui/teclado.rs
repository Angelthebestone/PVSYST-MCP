use super::{exigir_algun_foco, exigir_foco_en_la_ventana, resolver_foco, Destino};
use crate::error::Error;

/// BUG-23: `ESC` went out with `foco:false` and `estado:ok` and nothing received it.
#[test]
fn should_refuse_the_keys_when_no_control_holds_the_focus_on_the_user_desktop() {
    let error = exigir_algun_foco(0, Destino::Usuario, 0x10).expect_err("nowhere to deliver");

    assert!(error.to_string().contains("0x10"), "{error}");
    assert!(exigir_algun_foco(0, Destino::Aislado, 0x10).is_ok());
    assert!(exigir_algun_foco(0x20, Destino::Usuario, 0x10).is_ok());
}

#[test]
fn should_refuse_to_send_the_keys_when_the_control_did_not_take_the_focus_on_the_user_desktop() {
    let resultado = resolver_foco(false, Destino::Usuario, 0x10);

    assert!(matches!(resultado, Err(Error::Gui(m)) if m.contains("0x10")));
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
