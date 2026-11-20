//! `pvsyst://gui/<clase>`: one window of the GUI map instead of all of them.

use super::*;
use crate::adapters::gui::catalogo::mapa_de_ventanas;

/// A configuration that names no PVsyst install and reads no environment
/// variable, so no assertion depends on the machine the tests run on.
fn paths_de_prueba() -> Paths {
    Paths::para_test("C:/noexiste/ws")
}

/// What the `metadatos` block must carry, read from `metadatos()` in
/// `src/adapters/gui/catalogo/recurso.rs`. `aviso` is always there (null when
/// there is nothing to warn about), so its presence alone proves nothing.
fn comprobar_metadatos(ventana: &Value) {
    let mapa = mapa_de_ventanas();
    let metadatos = &ventana["metadatos"];
    assert_eq!(
        metadatos["measured_on"].as_str(),
        mapa["medido_el"].as_str(),
        "the block says when the map was measured"
    );
    assert_eq!(
        metadatos["pvsyst_version"].as_str(),
        mapa["version_pvsyst"]
            .as_str()
            .and_then(|v| v.split_whitespace().next()),
        "the block names the version the map was measured on"
    );
    assert_eq!(metadatos["idioma_medido"].as_str(), mapa["idioma"].as_str());
    assert_eq!(
        metadatos["windows_total"].as_u64(),
        mapa["ventanas"]
            .as_object()
            .map(|ventanas| ventanas.len() as u64),
        "the block counts every window of the map"
    );
    assert!(
        metadatos["pvsyst_version_en_ejecucion"].is_null(),
        "the test paths name no install, so there is no version to compare: {metadatos}"
    );
    let cobertura = &metadatos["cobertura"];
    assert!(
        cobertura["ventanas_con_campos"]
            .as_u64()
            .is_some_and(|n| n > 0),
        "the block says how much of the map carries fields: {cobertura}"
    );
    assert_eq!(
        cobertura["controles_sin_hwnd"].as_u64(),
        mapa["controles_sin_hwnd"]
            .as_array()
            .map(|controles| controles.len() as u64),
        "the block counts the controls the map records as unreachable"
    );
    // Only the language half of the warning can fire here, and only when this
    // machine runs a PVsyst in a language other than the map's: the version half
    // needs an install, and these paths name none. The texts are VERSION_DISTINTA
    // and IDIOMA_DISTINTO in `src/adapters/gui/catalogo/recurso.rs`.
    let aviso = metadatos["aviso"].as_str().unwrap_or_default();
    assert!(
        !aviso.contains("may have changed"),
        "version warning without a version: {aviso}"
    );
    assert!(
        aviso.is_empty() || aviso.contains("will not match"),
        "the only warning left here is the language one: {aviso}"
    );
}

/// The body a client receives for `uri`.
fn texto(uri: &str, paths: &Paths) -> String {
    let valor = read(uri, paths).expect("resource");
    valor["contents"][0]["text"]
        .as_str()
        .expect("text")
        .to_owned()
}

fn cuerpo(uri: &str, paths: &Paths) -> Value {
    serde_json::from_str(&texto(uri, paths)).expect("json")
}

#[test]
fn should_serve_one_window_with_the_functions_that_open_it() {
    let paths = paths_de_prueba();
    let ventana = cuerpo("pvsyst://gui/TDetailedLossesForm", &paths);

    assert_eq!(ventana["clase"], "TDetailedLossesForm");
    let entrada = mapa_de_ventanas()
        .get("ventanas")
        .and_then(|ventanas| ventanas.get("TDetailedLossesForm"))
        .expect("window in the map");
    assert_eq!(&ventana["ventana"], entrada);
    let abren: Vec<&str> = ventana["abierta_con"]
        .as_array()
        .expect("abierta_con")
        .iter()
        .filter_map(Value::as_str)
        .collect();
    assert!(abren.contains(&"variante.perdidas_detalladas"), "{abren:?}");
    comprobar_metadatos(&ventana);
    assert!(ventana.get("mapa").is_none(), "the whole map came along");

    let minusculas = cuerpo("pvsyst://gui/tdetailedlossesform", &paths);
    assert_eq!(minusculas["clase"], "TDetailedLossesForm");

    let una = texto("pvsyst://gui/TDetailedLossesForm", &paths);
    let completa = texto("pvsyst://gui", &paths);
    assert!(
        una.len() * 2 < completa.len(),
        "one window {} B, the whole map {} B",
        una.len(),
        completa.len()
    );
}

#[test]
fn should_list_the_map_classes_when_the_window_is_unknown() {
    let paths = paths_de_prueba();
    let error = read("pvsyst://gui/TNoExisteForm", &paths).expect_err("unknown window");

    assert!(matches!(&error, Error::NotFound(_)), "{error}");
    let mensaje = error.to_string();
    assert!(mensaje.contains("TNoExisteForm"), "{mensaje}");
    assert!(mensaje.contains("TDetailedLossesForm"), "{mensaje}");
}

#[test]
fn should_keep_the_whole_map_in_the_gui_resource() {
    let paths = paths_de_prueba();
    let completo = cuerpo("pvsyst://gui", &paths);

    assert_eq!(&completo["mapa"], mapa_de_ventanas());
    assert!(completo["funciones"]
        .as_array()
        .is_some_and(|funciones| !funciones.is_empty()));
    let uso = completo["uso"].as_str().expect("uso");
    assert!(uso.contains("pvsyst://gui/<"), "{uso}");
}

#[test]
fn should_advertise_the_per_window_form_as_a_resource_template() {
    let servidor = crate::Server::new(paths_de_prueba());
    let respuesta = servidor
        .handle_line(r#"{"jsonrpc":"2.0","id":1,"method":"resources/templates/list"}"#)
        .expect("response");

    let sobre: Value = serde_json::from_str(&respuesta).expect("json");
    assert_eq!(
        sobre["result"]["resourceTemplates"][0]["uriTemplate"],
        "pvsyst://gui/{ventana}"
    );
}
