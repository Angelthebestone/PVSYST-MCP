use super::*;
use crate::idioma::Idioma;

/// BUG-17: with the home screen open `vista` said no window was open.
#[test]
fn should_name_the_open_windows_that_have_no_schema() {
    let texto = texto_sin_entidad(
        &["THomeForm".to_string(), "TDimensForm".to_string()],
        Idioma::Es,
    );

    assert!(texto.contains("THomeForm, TDimensForm"), "{texto}");
    assert!(texto.contains("ventanas abiertas"), "{texto}");
}

#[test]
fn should_say_nothing_is_open_when_no_window_is_open() {
    assert!(texto_sin_entidad(&[], Idioma::Es).contains("ninguna ventana de PVsyst abierta"));
}
