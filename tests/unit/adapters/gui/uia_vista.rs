use super::*;

#[test]
fn operacion_para_picks_the_natural_pattern() {
    assert_eq!(operacion_para(Some("Checkbox")), "alternar");
    assert_eq!(operacion_para(Some("tab item")), "seleccionar");
    assert_eq!(operacion_para(Some("tree item")), "expandir");
    assert_eq!(operacion_para(Some("button")), "invocar");
    assert_eq!(operacion_para(None), "invocar");
}

#[test]
fn fusionar_merges_by_hwnd_and_flags_visual_only_regions() {
    let arbol = json!({
        "nombre": "root",
        "hijos": [
            {"nombre": "Herramientas", "rol": "button", "patrones": ["invoke"],
             "rect": [1, 2, 3, 4], "hijos": [], "hwnd": "0x10"},
            {"nombre": "panel", "rol": "pane", "patrones": [],
             "rect": [0, 0, 5, 5], "hijos": []},
        ],
    });
    let win = vec![
        json!({"hwnd": "0x10", "class": "TButton", "title": "", "visible": true}),
        json!({"hwnd": "0x20", "class": "TPanel", "title": "", "visible": true}),
    ];
    let out = fusionar(win, &arbol);
    assert_eq!(out["ventanas"][0]["uia"]["rol"], "button");
    assert!(out["ventanas"][1].get("uia").is_none());
    let sin = out["uia_sin_hwnd"].as_array().unwrap();
    assert_eq!(sin.len(), 1);
    assert_eq!(sin[0]["nombre"], "panel");
    assert_eq!(sin[0]["solo_vision"], true);
}
