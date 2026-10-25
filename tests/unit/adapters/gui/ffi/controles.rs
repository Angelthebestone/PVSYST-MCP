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
    assert!(!elegir_en_combo(0, "item").unwrap());
    assert!(!elegir_en_lista(0, "item").unwrap());
    assert!(!elegir_item_por_clase(0, "TListBox", "item").unwrap());
    assert!(!elegir_item_por_clase(0, "TComboBox", "item").unwrap());
}

#[test]
fn should_fail_when_typing_or_changing_page_on_invalid_handle() {
    assert!(teclear_en(0, "test").is_err());
    assert!(mostrar_pagina(0, 2).is_err());
}

#[test]
fn should_accept_a_tab_index_inside_the_page_count() {
    assert!(exigir_indice_pagina(0, 3, "Formulario").is_ok());
    assert!(exigir_indice_pagina(2, 3, "Formulario").is_ok());
}

#[test]
fn should_refuse_a_tab_index_past_the_last_page_naming_the_count() {
    let err = exigir_indice_pagina(9, 3, "Formulario")
        .unwrap_err()
        .to_string();
    assert!(err.contains("'indice' 9 fuera de rango"), "{err}");
    assert!(err.contains("tiene 3 pestañas (0 a 2)"), "{err}");
}

#[test]
fn should_accept_a_tab_change_that_landed() {
    assert!(exigir_cambio_de_pestana(2, Some(2)).is_ok());
}

#[test]
fn should_refuse_a_tab_change_the_form_vetoed_naming_both_tabs() {
    let err = exigir_cambio_de_pestana(2, Some(0))
        .unwrap_err()
        .to_string();
    assert!(err.contains("no cambió a la pestaña 2"), "{err}");
    assert!(err.contains("sigue en la 0"), "{err}");
}
