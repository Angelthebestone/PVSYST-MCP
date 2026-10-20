use super::*;

#[test]
fn should_map_known_and_unknown_control_types() {
    assert_eq!(rol_de(50000), "button");
    assert_eq!(rol_de(50019), "tab item");
    assert_eq!(rol_de(1), "otro");
}

#[test]
fn should_preserve_com_error_when_reading_string() {
    let err: windows::core::Result<BSTR> = Err(windows::core::Error::empty());
    let res = cadena(err);
    assert!(
        res.is_err(),
        "Must preserve error instead of returning empty string"
    );
}
