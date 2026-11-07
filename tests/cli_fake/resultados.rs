use super::cliente_mcp::{fixture, synthetic_workspace, tool_text, Client};
use serde_json::json;
use std::fs;
use std::path::Path;

const FAKE: &str = env!("CARGO_BIN_EXE_fake_pvsystcli");

fn workspace() -> tempfile::TempDir {
    let ws = synthetic_workspace();
    fs::create_dir_all(ws.path().join("Results")).unwrap();
    ws
}

fn fake_env() -> Vec<(&'static str, &'static str)> {
    vec![
        ("PVSYST_CLI", FAKE),
        (
            "FAKE_PVSYST_FIXTURES",
            concat!(env!("CARGO_MANIFEST_DIR"), "/tests/fixtures/cli"),
        ),
    ]
}

fn ws_str(ws: &Path) -> String {
    ws.to_string_lossy().into_owned()
}

#[test]
fn should_snapshot_previous_deliverable_when_regenerated() {
    let ws = workspace();
    let resultados = ws.path().join("Results");
    for nombre in ["MCP_E2E.csv", "MCP_E2E.pdf"] {
        fs::copy(
            fixture(&format!("results/{nombre}")),
            resultados.join(nombre),
        )
        .unwrap();
    }
    let mut client = Client::start_env(Some(&ws_str(ws.path())), &fake_env());
    client.initialize();
    let args = json!({
        "que": "entregable",
        "proyecto": "MCP_E2E",
        "archivo": resultados.join("MCP_E2E.csv").to_string_lossy(),
    });

    let primero = tool_text(&client.call(2, "resultados", args.clone()));
    let segundo = tool_text(&client.call(3, "resultados", args));

    let creacion = primero["historial_id"]
        .as_str()
        .unwrap_or_else(|| panic!("la creación del entregable deja historial: {primero}"));
    let regeneracion = segundo["historial_id"]
        .as_str()
        .unwrap_or_else(|| panic!("la regeneración deja historial: {segundo}"));
    assert_ne!(
        creacion, regeneracion,
        "cada escritura tiene su propio snapshot"
    );
}
