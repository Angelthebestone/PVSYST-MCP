//! A written `.HOR` lands in `Shadings/` under its own name, not under the folder
//! the caller happened to write in the argument.

use super::cliente_mcp::{synthetic_workspace, tool_text, Client};
use serde_json::json;

#[test]
fn should_write_the_horizon_under_shadings_by_file_name() {
    let ws = synthetic_workspace();
    let ws_str = ws.path().to_string_lossy().to_string();
    let mut c = Client::start(Some(&ws_str));
    c.initialize();

    let resp = c.call(
        76,
        "sitio",
        json!({
            "accion": "escribir_horizonte", "archivo": "Sites/QA_Horizonte.HOR",
            "puntos": [
                { "azimut": -90, "altura": 3 }, { "azimut": 0, "altura": 10 },
                { "azimut": 90, "altura": 5 }, { "azimut": 180, "altura": 0 },
            ],
        }),
    );
    let out = tool_text(&resp);
    let archivo = out["archivo"].as_str().unwrap_or_default();
    assert!(
        archivo.ends_with("Shadings/QA_Horizonte.HOR")
            || archivo.ends_with(r"Shadings\QA_Horizonte.HOR"),
        "el .HOR queda en Shadings con su nombre: {archivo}"
    );
    assert!(
        !ws.path().join("Shadings").join("Sites").exists(),
        "no se crea una carpeta Sites dentro de Shadings"
    );
}
