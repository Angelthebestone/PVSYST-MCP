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

#[test]
fn should_map_every_function_key_to_consecutive_virtual_keys() {
    let codigos: Vec<Option<u8>> = (1..=12).map(|n| vk_for(&format!("F{n}"))).collect();
    let esperado: Vec<Option<u8>> = (0x70..=0x7B).map(Some).collect();
    assert_eq!(codigos, esperado);
    assert_eq!(vk_for("F13"), None);
    assert_eq!(vk_for("F0"), None);
}

#[test]
fn should_send_home_end_and_page_keys_to_move_through_a_list() {
    assert_eq!(vk_for("HOME"), Some(0x24));
    assert_eq!(vk_for("end"), Some(0x23));
    assert_eq!(vk_for("PGUP"), vk_for("PageUp"));
    assert_eq!(vk_for("PGDN"), Some(0x22));
}

#[test]
fn should_refuse_delete_and_insert_even_with_modifiers() {
    for tecla in [
        "DELETE",
        "del",
        "Supr",
        "INSERT",
        "ins",
        "SHIFT+DELETE",
        "CTRL+INSERT",
    ] {
        let err = parsear_combinacion(tecla).unwrap_err().to_string();
        assert!(err.contains("no se envía"), "{tecla}: {err}");
    }
}

#[test]
fn should_keep_letters_digits_and_the_keys_that_already_worked() {
    assert_eq!(vk_for("a"), Some(b'A'));
    assert_eq!(vk_for("7"), Some(b'7'));
    assert_eq!(vk_for("Enter"), Some(0x0D));
    assert_eq!(vk_for("down"), Some(0x28));
    assert_eq!(vk_for("ñ"), None);
}
