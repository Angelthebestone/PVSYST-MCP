use super::*;

#[test]
fn should_return_empty_text_when_control_handle_is_invalid() {
    assert_eq!(texto_control(0), "");
    assert_eq!(texto_control(-1), "");
}
