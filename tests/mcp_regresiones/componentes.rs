//! `componente crear` copies the template when the call asks for no changes.

use super::cliente_mcp::{synthetic_workspace, tool_text, Client};
use serde_json::json;
use std::fs;

#[test]
fn should_copy_the_template_byte_for_byte_when_componente_crear_gets_no_cambios() {
    let ws = synthetic_workspace();
    let plantilla = ws
        .path()
        .join("ComposPV")
        .join("PVModules")
        .join("Synth_400W.PAN");
    let nuevo = ws
        .path()
        .join("ComposPV")
        .join("PVModules")
        .join("QA_PAN_450.PAN");
    let ws_str = ws.path().to_string_lossy().to_string();
    let mut c = Client::start(Some(&ws_str));
    c.initialize();

    let resp = c.call(
        75,
        "componente",
        json!({
            "accion": "crear", "plantilla": "Synth_400W",
            "nombre_nuevo": "QA_PAN_450", "tipo": "PAN",
        }),
    );
    let out = tool_text(&resp);
    assert_eq!(out["changes_applied"], json!([]), "resp={out}");
    assert_eq!(
        fs::read(&nuevo).expect("componente creado"),
        fs::read(&plantilla).expect("plantilla"),
        "la plantilla se copia tal cual"
    );
}
