use super::*;

#[test]
fn should_split_a_modifier_and_one_principal_key() {
    let (mods, _vk) = parsear_combinacion("CTRL+S").expect("valid combo");
    assert_eq!(mods, vec![VK_CONTROL]);
}

#[test]
fn should_refuse_an_unknown_token_naming_it() {
    let err = parsear_combinacion("ZZZ+Q").unwrap_err().to_string();
    assert!(err.contains("Tecla no reconocida 'ZZZ'"), "{err}");
}

#[test]
fn should_refuse_a_combo_with_two_principal_keys() {
    let err = parsear_combinacion("CTRL+S+Q").unwrap_err().to_string();
    assert!(err.contains("exactamente una tecla principal"), "{err}");
}
