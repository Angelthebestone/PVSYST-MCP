//! The language of PVsyst's windows: what the ini and the open windows say, and
//! the warning the caller gets when captions cannot be trusted.

use super::{nombre_en_ini, Lectura};
use crate::adapters::gui::rotulos::IdiomaGui;
use crate::idioma::Idioma;

/// The two `Language` keys PVsyst's ini has: the one that counts is the dialogs'.
const INI: &str = "[Print]\r\nLanguage=English\r\n[Preferences]\r\nDialog Language=Spanish\r\n";

fn lectura(ini: Option<&str>, ventanas: Option<IdiomaGui>) -> Lectura {
    Lectura {
        ini: ini.map(str::to_string),
        ventanas,
    }
}

#[test]
fn should_read_the_dialog_language_and_not_the_print_language_of_the_ini() {
    assert_eq!(nombre_en_ini(INI).as_deref(), Some("Spanish"));
    assert_eq!(nombre_en_ini("[Print]\r\nLanguage=English\r\n"), None);
    assert_eq!(nombre_en_ini("[Preferences]\r\nDialog Language=\r\n"), None);
}

#[test]
fn should_trust_the_open_windows_over_the_ini_and_say_when_they_differ() {
    let respuesta = lectura(Some("Spanish"), Some(IdiomaGui::En)).como_json(Idioma::En);

    assert_eq!(respuesta["idioma_gui"], "en");
    assert_eq!(respuesta["fuente"], "ventanas");
    assert_eq!(respuesta["nombre_en_ini"], "Spanish");
    let aviso = respuesta["aviso"].as_str().unwrap_or_default();
    assert!(
        aviso.contains("'Spanish'") && aviso.contains("'en'"),
        "{aviso}"
    );
}

#[test]
fn should_read_the_language_from_the_ini_when_no_window_is_open() {
    let respuesta = lectura(Some("Spanish"), None).como_json(Idioma::En);

    assert_eq!(respuesta["idioma_gui"], "es");
    assert_eq!(respuesta["fuente"], "PVsyst.ini");
    // Spanish is the language the captions were measured in: nothing to warn about.
    assert_eq!(respuesta["aviso"], serde_json::Value::Null);
    assert_eq!(
        respuesta["idiomas_con_rotulos"],
        serde_json::json!(["es", "en", "fr", "de"])
    );
}

#[test]
fn should_warn_that_captions_are_inferred_for_a_language_never_measured() {
    let respuesta = lectura(Some("English"), None).como_json(Idioma::Es);

    assert_eq!(respuesta["idioma_gui"], "en");
    let aviso = respuesta["aviso"].as_str().unwrap_or_default();
    assert!(
        aviso.contains("inferidos") && aviso.contains("hwnd"),
        "{aviso}"
    );
}

#[test]
fn should_name_the_language_the_captions_do_not_cover_and_the_way_around_it() {
    let respuesta = lectura(Some("Italian"), None).como_json(Idioma::En);

    assert_eq!(respuesta["idioma_gui"], serde_json::Value::Null);
    assert_eq!(
        lectura(Some("Italian"), None).codigo_o_nombre().as_deref(),
        Some("Italian")
    );
    let aviso = respuesta["aviso"].as_str().unwrap_or_default();
    assert!(
        aviso.contains("'Italian'") && aviso.contains("que:inventario"),
        "{aviso}"
    );
    // The language of the MCP's messages is a different thing and is reported apart.
    assert_eq!(respuesta["idioma_mensajes_mcp"], "en");
}
