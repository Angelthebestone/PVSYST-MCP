use super::*;

#[test]
fn should_keep_bom_and_accents_when_roundtripping_utf16() {
    let texto = "[Preferences]\r\nIdioma=Español\r\n";
    let bytes = codificar(texto, Codificacion::Utf16Le).expect("utf16");
    assert!(bytes.starts_with(BOM_UTF16LE));
    let (leido, cod) = decodificar(&bytes);
    assert_eq!(cod, Codificacion::Utf16Le);
    assert_eq!(leido, texto);
}

#[test]
fn should_mask_secrets_when_the_section_or_key_name_is_protected() {
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
