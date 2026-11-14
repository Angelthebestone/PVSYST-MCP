use super::*;
use crate::adapters::gui::ventana_falsa::{Hijo, VentanaFalsa};
use crate::config::Paths;

const PAGINA: (i32, i32, i32, i32) = (0, 20, 780, 560);

/// A form with a shown page (a combo and a button) and a hidden one (a ticked
/// option, an edit and a layout panel).
fn formulario_con_pagina_oculta() -> VentanaFalsa {
    VentanaFalsa::crear(
        "TSystemeAddForm",
        "Gestión de la energía",
        vec![
            Hijo::de("TPageControl", (0, 0, 780, 560)),
            Hijo::de("TTabSheet", PAGINA)
                .en(1)
                .con_texto("Conexión a la red"),
            Hijo::de("TTabSheet", PAGINA)
                .en(1)
                .con_texto("Estimación P50 - P90")
                .oculto(),
            Hijo::de("TComboBox", (20, 40, 200, 60)).en(2),
            Hijo::de("TPngBitBtn", (600, 520, 700, 550)).con_texto("OK"),
            Hijo::de("TCheckBox", (20, 30, 300, 50))
                .en(3)
                .con_texto("Activo")
                .oculto(),
            Hijo::de("TEdit", (320, 30, 400, 50))
                .en(3)
                .con_texto("95")
                .oculto(),
            Hijo::de("TGridPanel", (0, 80, 400, 160)).en(3).oculto(),
        ],
    )
}

fn formulario_de(form: &VentanaFalsa, todos: bool) -> Value {
    let paths = Paths::para_test(std::env::temp_dir());
    let args = json!({
        "ventana": form.referencia(0),
        "pids": [std::process::id()],
        "ocr": false,
        "todos": todos,
    });
    formulario(&paths, &args).unwrap()
}

/// Classes of a list, in alphabetical order: the enumeration follows the Z-order.
fn clases(lista: &Value) -> Vec<&str> {
    let mut clases: Vec<&str> = lista
        .as_array()
        .unwrap()
        .iter()
        .map(|c| c["clase"].as_str().unwrap())
        .collect();
    clases.sort_unstable();
    clases
}

fn de_clase<'a>(lista: &'a Value, clase: &str) -> &'a Value {
    lista
        .as_array()
        .unwrap()
        .iter()
        .find(|c| c["clase"] == clase)
        .unwrap()
}

#[test]
fn should_list_hidden_fields_and_unclassified_controls_when_todos_is_true() {
    let form = formulario_con_pagina_oculta();

    let vista = formulario_de(&form, true);

    assert_eq!(
        clases(&vista["campos"]),
        ["TCheckBox", "TComboBox", "TEdit"]
    );
    let opcion = de_clase(&vista["campos"], "TCheckBox");
    assert_eq!(opcion["visible"], false);
    assert_eq!(opcion["pestana"], "Estimación P50 - P90");
    assert_eq!(opcion["parent"], form.referencia(3));
    assert_eq!(opcion["etiqueta"], "Activo");
    assert_eq!(opcion["etiqueta_fuente"], "window_text");
    assert_eq!(de_clase(&vista["campos"], "TEdit")["valor"], "95");
    assert_eq!(clases(&vista["botones"]), ["TPngBitBtn"]);
    let otros = clases(&vista["otros"]);
    for clase in ["TPageControl", "TTabSheet", "TGridPanel"] {
        assert!(otros.contains(&clase), "{clase} falta en otros: {otros:?}");
    }
    let panel = de_clase(&vista["otros"], "TGridPanel");
    assert!(panel["hwnd"].is_string() && panel["rect"]["right"].is_i64());
    assert_eq!(panel["visible"], false);
}

#[test]
fn should_keep_listing_only_the_shown_controls_without_todos() {
    let form = formulario_con_pagina_oculta();

    let vista = formulario_de(&form, false);

    assert_eq!(clases(&vista["campos"]), ["TComboBox"]);
    assert_eq!(vista["campos"][0]["visible"], true);
    assert!(!clases(&vista["otros"]).contains(&"TGridPanel"));
}
