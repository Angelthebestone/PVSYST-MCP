use super::*;
use serde_json::json;

#[test]
fn should_read_tab_order_handles_rectangles_and_vision_only_regions_from_the_tree() {
    let arbol = json!({
        "nombre": "form", "rol": "window", "hijos": [
            { "nombre": "Datos", "rol": "tab item", "rect": [0, 0, 50, 20], "patrones": ["seleccion"], "hijos": [] },
            { "nombre": "Sistema", "rol": "tab item", "rect": [50, 0, 100, 20], "patrones": [], "hijos": [] },
            { "nombre": "OK", "rol": "button", "hwnd": "0x1F4", "rect": [600, 520, 700, 550], "patrones": ["invoke"], "hijos": [] },
            { "nombre": "", "rol": "pane", "rect": [10, 40, 200, 100], "patrones": [], "hijos": [] },
            { "nombre": "sin área", "rol": "text", "rect": [5, 5, 5, 5], "patrones": [], "hijos": [] },
        ]
    });

    let lectura = lectura_de(&arbol, 6);

    assert_eq!(lectura.pestanas, ["Datos", "Sistema"]);
    let boton = lectura.nodos.iter().find(|n| n.rol == "button").unwrap();
    assert_eq!(boton.hwnd, Some(0x1F4));
    assert_eq!(boton.rect.map(|r| (r.left, r.bottom)), Some((600, 550)));
    assert_eq!(boton.patrones, ["invoke"]);
    let panel = lectura.nodos.iter().find(|n| n.rol == "pane").unwrap();
    assert!(panel.solo_vision && panel.hwnd.is_none());
    let sin_area = lectura
        .nodos
        .iter()
        .find(|n| n.nombre == "sin área")
        .unwrap();
    assert!(sin_area.rect.is_none());
}

/// A leaf at the depth limit has no children because the tree was not walked
/// below it, not because it draws everything itself.
#[test]
fn should_not_call_an_element_at_the_depth_limit_vision_only_and_report_a_cut_tree() {
    let arbol = json!({
        "rol": "window", "truncado": true, "hijos": [
            { "nombre": "", "rol": "pane", "rect": [0, 0, 100, 100], "patrones": [], "hijos": [
                { "nombre": "", "rol": "pane", "rect": [0, 0, 50, 50], "patrones": [], "hijos": [] },
            ] },
        ]
    });

    let lectura = lectura_de(&arbol, 2);

    assert!(lectura.truncado);
    let a_distinta_altura: Vec<bool> = lectura.nodos.iter().map(|n| n.solo_vision).collect();
    assert_eq!(a_distinta_altura, [false, false]);
    assert!(lectura_de(&arbol, 3).nodos[1].solo_vision);
}
