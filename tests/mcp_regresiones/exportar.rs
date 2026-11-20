use super::cliente_mcp::{error_text, synthetic_workspace, Client};
use serde_json::json;

#[test]
fn should_reject_parent_segments_in_workspace_create_path() {
    let ws = synthetic_workspace();
    let ws_str = ws.path().to_string_lossy().to_string();
    let mut c = Client::start(Some(&ws_str));
    c.initialize();

    let resp = c.call(
        30,
        "workspace",
        json!({ "accion": "crear", "ruta": "../x_nope", "workspace": ws_str }),
    );
    let msg = error_text(&resp);
    assert!(msg.contains("'ruta' no admite '..'"), "{msg}");
}
