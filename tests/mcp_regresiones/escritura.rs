//! `escribir_archivo` replaces an existing file only when the caller asks for it.

use super::cliente_mcp::{error_text, synthetic_workspace, tool_text, Client};
use serde_json::json;
use std::fs;

#[test]
fn should_refuse_to_overwrite_a_file_unless_sobrescribir_is_true() {
    let ws = synthetic_workspace();
    let nota = ws.path().join("UserData").join("nota.txt");
    let ws_str = ws.path().to_string_lossy().to_string();
    let mut c = Client::start(Some(&ws_str));
    c.initialize();

    let creada = c.call(
        72,
        "escribir_archivo",
        json!({ "archivo": "UserData/nota.txt", "contenido": "hola\n" }),
    );
    assert_eq!(creada["result"]["isError"], json!(false), "resp={creada}");

    let rechazada = c.call(
        73,
        "escribir_archivo",
        json!({ "archivo": "UserData/nota.txt", "contenido": "otra vez\n" }),
    );
    let msg = error_text(&rechazada);
    assert!(msg.contains("sobrescribir"), "nombra la alternativa: {msg}");
    assert_eq!(
        fs::read_to_string(&nota).expect("nota"),
        "hola\n",
        "sin 'sobrescribir' el archivo no cambia"
    );

    let reemplazada = c.call(
        74,
        "escribir_archivo",
        json!({ "archivo": "UserData/nota.txt", "contenido": "otra vez\n", "sobrescribir": true }),
    );
    let out = tool_text(&reemplazada);
    assert_eq!(out["bytes_written"], json!(9), "resp={out}");
    assert_eq!(fs::read_to_string(&nota).expect("nota"), "otra vez\n");
}
