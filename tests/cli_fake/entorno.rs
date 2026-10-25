use super::cliente_mcp::{error_text, synthetic_workspace, tool_text, Client};
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

fn local_con_ini(contenido: &str) -> tempfile::TempDir {
    let local = tempfile::tempdir().unwrap();
    let ini_dir = local.path().join("PVsyst").join("8.1").join("Admin");
    fs::create_dir_all(&ini_dir).unwrap();
    fs::write(ini_dir.join("PVsyst.ini"), contenido).unwrap();
    local
}

#[test]
fn should_return_only_requested_key_of_a_preferences_section() {
    let ws = synthetic_workspace();
    let local = local_con_ini("[Opciones]\r\nIdioma=es\r\nOtra=1\r\n");
    let local_str = local.path().to_string_lossy().into_owned();
    let env = [("LOCALAPPDATA", local_str.as_str())];
    let mut client = Client::start_env(Some(&ws_str(ws.path())), &env);
    client.initialize();
    let args = json!({ "accion": "preferencias", "seccion": "Opciones", "clave": "idioma" });
    let v = tool_text(&client.call(3, "entorno", args));
    assert_eq!(v["secciones"]["Opciones"]["Idioma"], json!("es"), "v={v}");
    assert!(v["secciones"]["Opciones"]["Otra"].is_null(), "v={v}");
}

#[test]
fn should_reject_preferences_key_without_section() {
    let ws = synthetic_workspace();
    let local = local_con_ini("[Opciones]\r\nIdioma=es\r\n");
    let local_str = local.path().to_string_lossy().into_owned();
    let env = [("LOCALAPPDATA", local_str.as_str())];
    let mut client = Client::start_env(Some(&ws_str(ws.path())), &env);
    client.initialize();
    let args = json!({ "accion": "preferencias", "clave": "Idioma" });
    let msg = error_text(&client.call(4, "entorno", args));
    assert!(msg.contains("'clave' requiere 'seccion'"), "msg={msg}");
}
