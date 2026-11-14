use super::cliente_mcp::{error_text, synthetic_workspace, tool_text, Client};
use serde_json::json;
use std::fs;

#[test]
fn should_copy_a_binary_component_without_field_changes() {
    let ws = synthetic_workspace();
    let ws_str = ws.path().to_string_lossy().to_string();
    let carpeta = ws.path().join("ComposPV").join("PVModules");
    let legacy: [u8; 5] = [0x00, 0x01, 0x02, 0xFF, 0x00];
    fs::write(carpeta.join("Legacy.PAN"), legacy).expect("legacy");

    let mut c = Client::start(Some(&ws_str));
    c.initialize();
    let resp = c.call(
        21,
        "componente",
        json!({
            "accion": "crear",
            "plantilla": carpeta.join("Legacy.PAN").to_string_lossy(),
            "nombre_nuevo": "Copia",
            "tipo": "PAN",
            "cambios": {},
        }),
    );
    assert_eq!(resp["result"]["isError"], json!(false), "resp={resp}");
    let creado = tool_text(&resp)["created"]
        .as_str()
        .expect("created")
        .to_string();
    assert!(creado.ends_with("Copia.PAN"), "created={creado}");
    assert_eq!(fs::read(carpeta.join("Copia.PAN")).expect("copia"), legacy);
}

#[test]
fn should_reject_launching_gui_when_the_executable_is_missing() {
    let mut c = Client::start(None);
    c.initialize();

    let resp = c.call(105, "gui_actuar", json!({ "accion": "lanzar" }));
    assert_eq!(resp["result"]["isError"], true, "resp={resp}");
    let msg = error_text(&resp);
    assert!(
        msg.contains("PVsyst GUI not found") || msg.contains("nonexistent_pvsyst_gui.exe"),
        "expected missing GUI error, got: {msg}"
    );
}
