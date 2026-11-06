use super::{resolver_foco, Destino};
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
