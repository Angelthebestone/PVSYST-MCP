use super::*;

/// BUG-17: with the home screen open `vista` said no window was open.
#[test]
fn should_name_the_open_window_that_has_no_schema() {
    let texto = texto_sin_entidad(&["THomeForm".to_string()]);

    assert!(texto.contains("THomeForm no tiene esquema"), "{texto}");
    assert!(!texto.contains("ninguna ventana"), "{texto}");
}

#[test]
fn should_say_nothing_is_open_when_no_window_is_open() {
    assert!(texto_sin_entidad(&[]).contains("ninguna ventana"));
}
