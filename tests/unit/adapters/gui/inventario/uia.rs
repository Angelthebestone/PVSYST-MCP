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

    let lectura = lectura_de(&arbol);

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
