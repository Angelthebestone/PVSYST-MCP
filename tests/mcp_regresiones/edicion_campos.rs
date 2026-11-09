//! `editar campos` must not invent fields: a typo would write a key PVsyst does not read.

use super::cliente_mcp::{error_text, synthetic_workspace, tool_text, Client};
use serde_json::json;
use std::fs;

#[test]
fn should_refuse_a_field_the_object_does_not_have_unless_nuevo_is_true() {
    let ws = synthetic_workspace();
    let vc0 = ws.path().join("Projects").join("SYNTH_Plant.VC0");
    let antes = fs::read(&vc0).expect("VC0 del fixture");
    let ws_str = ws.path().to_string_lossy().to_string();
    let mut c = Client::start(Some(&ws_str));
    c.initialize();

    let rechazado = c.call(
        70,
        "editar",
        json!({
            "accion": "campos", "archivo": "Projects/SYNTH_Plant.VC0",
            "objeto": "pvSubArray", "campos": { "FieldTitl": "25" },
        }),
    );
    let msg = error_text(&rechazado);
    assert!(msg.contains("FieldTitl"), "nombra el campo: {msg}");
    assert!(msg.contains("nuevo"), "nombra la alternativa: {msg}");
    assert_eq!(
        fs::read(&vc0).expect("VC0 tras el rechazo"),
        antes,
        "un campo desconocido no toca el archivo"
    );

    let aceptado = c.call(
        71,
        "editar",
        json!({
            "accion": "campos", "archivo": "Projects/SYNTH_Plant.VC0",
            "objeto": "pvSubArray", "campos": { "FieldTitl": "25" }, "nuevo": true,
        }),
    );
    let out = tool_text(&aceptado);
    assert_eq!(out["changes"][0]["action"], json!("inserted"), "resp={out}");
    assert!(fs::read_to_string(&vc0)
        .expect("VC0")
        .contains("FieldTitl=25"));
}
