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
fn should_report_missing_economics_when_pdf_lacks_economic_evaluation() {
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

    let resp = client.call(
        2,
        "resultados",
        json!({
            "que": "economia",
            "proyecto": "MCP_E2E",
            "archivo": resultados.join("MCP_E2E.csv").to_string_lossy(),
        }),
    );

    assert_eq!(resp["result"]["isError"], true, "resp={resp}");
    let msg = resp["result"]["content"][0]["text"].as_str().unwrap();
    assert!(msg.contains("proyecto accion:economia"), "msg={msg}");
}

#[test]
fn should_report_missing_carbon_when_pdf_lacks_carbon_balance() {
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

    let resp = client.call(
        2,
        "resultados",
        json!({
            "que": "carbono",
            "proyecto": "MCP_E2E",
            "archivo": resultados.join("MCP_E2E.csv").to_string_lossy(),
        }),
    );

    assert_eq!(resp["result"]["isError"], true, "resp={resp}");
    let msg = resp["result"]["content"][0]["text"].as_str().unwrap();
    assert!(msg.contains("PrintInReport"), "msg={msg}");
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

    assert_eq!(primero["historial_id"], json!(null), "v={primero}");
    assert!(segundo["historial_id"].is_string(), "v={segundo}");
}
