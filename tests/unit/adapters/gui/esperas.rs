use super::*;
use serde_json::json;

#[test]
fn should_mark_the_answer_when_the_window_arrived_late() {
    let mut window = json!({"hwnd": "0x10", "class": "TProjectForm"});

    marcar_espera(
        &mut window,
        &Espera {
            tarde: true,
            timeout_ms: 12_000,
            etiqueta: "TProjectForm",
        },
    );

    assert_eq!(window["tarde"], true);
    assert_eq!(window["esperado_ms"], 12_000);
    assert!(
        window["aviso"]
            .as_str()
            .unwrap_or_default()
            .contains("12000 ms"),
        "{window}"
    );
}
