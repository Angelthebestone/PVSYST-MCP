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

#[test]
fn should_compare_variants_when_workspace_override_differs_from_default() {
    let origen = synthetic_workspace();
    let destino = workspace();
    let projects = destino.path().join("Projects");
    fs::create_dir_all(&projects).unwrap();
    let base_vc = origen.path().join("Projects").join("SYNTH_Plant.VC0");
    let cabecera = fs::read_to_string(fixture("results/MCP_E2E.csv")).unwrap();
    let resultados = destino.path().join("Results");
    for v in ["VC0", "VC1"] {
        fs::copy(&base_vc, projects.join(format!("MCP_E2E.{v}"))).unwrap();
        fs::write(
            resultados.join(format!("MCP_E2E_{v}.csv")),
            cabecera.replace("MCP_E2E.VC0", &format!("MCP_E2E.{v}")),
        )
        .unwrap();
        fs::copy(
            fixture("results/MCP_E2E.pdf"),
            resultados.join(format!("MCP_E2E_{v}.pdf")),
        )
        .unwrap();
    }
    let mut client = Client::start_env(Some(&ws_str(origen.path())), &fake_env());
    client.initialize();

    let resp = client.call(
        2,
        "resultados",
        json!({
            "que": "comparar",
            "proyecto": "MCP_E2E",
            "variantes": ["VC0", "VC1"],
            "workspace": ws_str(destino.path()),
        }),
    );

    let msg = resp["result"]["content"][0]["text"].as_str().unwrap();
    assert!(!msg.contains("fuera del workspace"), "msg={msg}");
    assert_ne!(resp["result"]["isError"], true, "msg={msg}");
}
