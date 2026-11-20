//! `pvsyst://gui/<clase>`: one window of the GUI map instead of all of them.

use super::*;
use crate::adapters::gui::catalogo::mapa_de_ventanas;

/// A configuration that finds no PVsyst install, so no assertion depends on the
/// machine the tests run on.
fn paths_de_prueba() -> Paths {
    let mut paths = Paths::resolve();
    paths.gui_exe = std::path::PathBuf::from("C:/noexiste/PVsyst.exe");
    paths
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
    assert!(
        ventana["metadatos"].get("aviso").is_some(),
        "the metadatos block lost aviso"
    );
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
