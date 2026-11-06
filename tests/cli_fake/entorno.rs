use super::cliente_mcp::{synthetic_workspace, tool_text, Client};
use serde_json::json;
use std::fs;
use std::path::Path;

fn ws_str(ws: &Path) -> String {
    ws.to_string_lossy().into_owned()
}

#[test]
fn should_snapshot_preferences_outside_workspace_when_editing() {
    let ws = synthetic_workspace();
    fs::create_dir_all(ws.path().join("Results")).unwrap();
    let local = tempfile::tempdir().unwrap();
    let ini_dir = local.path().join("PVsyst").join("8.1").join("Admin");
    fs::create_dir_all(&ini_dir).unwrap();
    let ini = ini_dir.join("PVsyst.ini");
    let original = "[Opciones]\r\nIdioma=es\r\n";
    fs::write(&ini, original).unwrap();
    let local_str = local.path().to_string_lossy().into_owned();
    let env = [("LOCALAPPDATA", local_str.as_str())];
    let mut client = Client::start_env(Some(&ws_str(ws.path())), &env);
    client.initialize();
    let args = json!({
        "accion": "preferencias_editar",
        "seccion": "Opciones",
        "clave": "Idioma",
        "valor": "en",
    });
    let v = tool_text(&client.call(2, "entorno", args));
    let historial = Path::new(v["historial_dir"].as_str().expect("historial_dir"));
    assert!(historial.starts_with(local.path()));
    assert_eq!(
        fs::read_to_string(historial.join("000.bin")).unwrap(),
        original
    );
    assert!(!ws.path().join("UserData").join("mcp_history").exists());
}
