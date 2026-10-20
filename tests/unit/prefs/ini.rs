use super::*;

#[test]
fn utf16_round_trip_keeps_bom_and_accents() {
    let texto = "[Preferences]\r\nIdioma=Español\r\n";
    let bytes = codificar(texto, Codificacion::Utf16Le);
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
