use super::*;

#[test]
fn utf16_round_trip_keeps_bom_and_accents() {
    let texto = "[Preferences]\r\nIdioma=Español\r\n";
    let bytes = codificar(texto, Codificacion::Utf16Le).expect("utf16");
    assert!(bytes.starts_with(BOM_UTF16LE));
    let (leido, cod) = decodificar(&bytes);
    assert_eq!(cod, Codificacion::Utf16Le);
    assert_eq!(leido, texto);
}

#[test]
fn secrets_are_masked_in_protected_sections_and_key_names() {
    assert!(oculto("API", "Ruta"));
    assert!(oculto("Web", "Puerto"));
    assert!(oculto("Preferences", "ApiToken"));
    assert!(oculto("Preferences", "db_Password"));
    assert!(!oculto("Preferences", "Idioma"));
    let secciones = parsear("[Preferences]\r\nIdioma=es\r\nApiToken=abc\r\n[Web]\r\nPuerto=80\r\n");
    assert_eq!(secciones["Preferences"]["Idioma"], "es");
    assert_eq!(secciones["Preferences"]["ApiToken"], "***");
    assert_eq!(secciones["Web"]["Puerto"], "***");
}

#[test]
fn reemplazar_clave_changes_only_the_target_section() {
    let texto = "[Preferences]\r\n  Idioma=es\r\nIdioma=x\r\n[Directory]\r\nIdioma=y\r\n";
    let nuevo = reemplazar_clave(texto, "Preferences", "idioma", "en").unwrap();
    assert_eq!(
        nuevo,
        "[Preferences]\r\n  Idioma=en\r\nIdioma=x\r\n[Directory]\r\nIdioma=y\r\n"
    );
}

#[test]
fn reemplazar_clave_fails_when_key_is_missing() {
    let err = reemplazar_clave("[Preferences]\r\nA=1\r\n", "Preferences", "B", "2").unwrap_err();
    assert!(err.to_string().contains("no existe"));
}

#[test]
fn should_round_trip_an_ansi_ini_byte_for_byte() {
    let bytes = b"[Preferences]\r\nIdioma=Espa\xf1ol\r\n".to_vec();

    let (texto, cod) = decodificar_sin_perdida(&bytes).expect("ansi");

    assert_eq!(cod, Codificacion::Latin1);
    assert!(texto.contains("Español"));
    assert_eq!(codificar(&texto, cod).expect("ansi"), bytes);
}

#[test]
fn should_refuse_to_edit_an_ini_that_would_lose_bytes() {
    let mut utf16_impar = BOM_UTF16LE.to_vec();
    utf16_impar.extend_from_slice(&[b'[', 0, b'P']);

    let res = decodificar_sin_perdida(&utf16_impar);

    assert!(matches!(res, Err(crate::error::Error::Format(_))));
}

#[test]
fn should_refuse_a_value_outside_latin1_in_an_ansi_ini() {
    let res = codificar("Clave=日本", Codificacion::Latin1);

    assert!(matches!(res, Err(crate::error::Error::Invalid(_))));
}
