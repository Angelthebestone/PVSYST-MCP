use super::*;
use crate::adapters::gui::observar;
use crate::adapters::gui::ventana_falsa::{Hijo, VentanaFalsa};
use serde_json::json;

/// The tree UI Automation gives a `TVirtualStringTree`: rows with no window handle.
#[test]
fn should_list_the_rows_of_a_tree_with_name_rect_and_index() {
    let arbol = json!({
        "rol": "tree", "hijos": [
            { "nombre": "Proyecto A", "rol": "tree item", "rect": [12, 40, 380, 59], "patrones": ["seleccion", "virtualizado"], "hijos": [] },
            { "nombre": "Proyecto B", "rol": "tree item", "rect": [12, 59, 380, 78], "patrones": ["seleccion"], "hijos": [] },
            { "nombre": "", "rol": "pane", "rect": [0, 0, 0, 0], "patrones": [], "hijos": [] },
        ]
    });

    let elementos = elementos_de(&uia::lectura_de(&arbol));

    let vista: Vec<_> = elementos
        .iter()
        .map(|e| (e.indice, e.nombre.as_str(), e.rect.top))
        .collect();
    assert_eq!(vista, [(0, "Proyecto A", 40), (1, "Proyecto B", 59)]);
    assert!(elementos[0].patrones.contains(&"virtualizado".to_string()));
    assert!(elementos.iter().all(|e| e.hwnd.is_none()));
}

/// A control that UI Automation does not open: the answer says it is vision-only
/// and names what still reaches it, instead of an empty list that looks like "no rows".
#[test]
fn should_flag_a_control_without_ui_automation_elements_as_vision_only() {
    let arbol = VentanaFalsa::crear(
        "TProjectLoadForm",
        "Cargar",
        vec![Hijo::de("TVirtualStringTree", (10, 40, 380, 200))],
    );
    let paths = Paths::para_test(std::env::temp_dir());
    let args = json!({
        "que": "elementos",
        "ventana": arbol.referencia(1),
        "pids": [std::process::id()],
    });

    let r = observar(&paths, &args).unwrap();

    assert_eq!(
        (r["solo_vision"].as_bool(), r["total"].as_u64()),
        (Some(true), Some(0))
    );
    assert_eq!(r["clase"], "TVirtualStringTree");
    assert!(r["alternativas"].as_str().unwrap().contains("accion:fila"));
}
