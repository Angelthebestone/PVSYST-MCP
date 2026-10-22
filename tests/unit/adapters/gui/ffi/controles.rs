use super::*;

#[test]
fn should_return_empty_string_when_control_handle_is_invalid() {
    assert_eq!(texto_control(0), "");
    assert_eq!(texto_control(-1), "");
}

#[test]
fn should_return_empty_options_when_combo_handle_is_invalid() {
    assert!(opciones_de_combo(0).is_empty());
    assert!(opciones_de_combo(-1).is_empty());
}

#[test]
fn should_return_false_when_selecting_item_on_invalid_handle() {
    assert!(!elegir_en_combo(0, "item"));
    assert!(!elegir_en_lista(0, "item"));
    assert!(!elegir_item_por_clase(0, "TListBox", "item"));
    assert!(!elegir_item_por_clase(0, "TComboBox", "item"));
}

#[test]
fn should_not_panic_when_typing_or_changing_page_on_invalid_handle() {
    teclear_en(0, "test");
    mostrar_pagina(0, 2);
}
