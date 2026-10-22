//! End-to-end tests of the CLI runner against `fake_pvsystcli`, which replays
//! recorded PVsystCLI stdout. No PVsyst licence or trial execution is used.

// Test code: failing loudly on unexpected state is the point.
#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::too_many_lines,
    clippy::needless_pass_by_value,
    unreachable_pub
)]

mod common;

use common::{fixture, synthetic_workspace, tool_text, Client};
use serde_json::json;
use std::fs;
use std::path::Path;
use std::time::Duration;

const FAKE: &str = env!("CARGO_BIN_EXE_fake_pvsystcli");

/// Private copy of the synthetic workspace (project `SYNTH_Plant`).
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
fn help_succeeds_and_unknown_command_fails() {
    let ws = workspace();
    let mut client = Client::start_env(Some(&ws_str(ws.path())), &fake_env());
    client.initialize();
    let ok = tool_text(&client.call(2, "pvsyst_cli", json!({ "comando": "help" })));
    assert_eq!(ok["success"], json!(true));
    let bad = tool_text(&client.call(3, "pvsyst_cli", json!({ "comando": "nocommand" })));
    assert_eq!(bad["success"], json!(false));
}

#[test]
fn slow_command_is_killed_at_timeout() {
    let ws = workspace();
    let mut env = fake_env();
    env.push(("FAKE_PVSYST_SLEEP_MS", "5000"));
    let mut client = Client::start_env(Some(&ws_str(ws.path())), &env);
    client.initialize();
    let v = tool_text(&client.call(
        2,
        "pvsyst_cli",
        json!({ "comando": "help", "timeout_s": 1 }),
    ));
    assert_eq!(v["success"], json!(false));
    assert_eq!(v["detected_error"], json!("timeout tras 1 s"));
}

#[test]
fn simulation_reports_success_from_fixture() {
    let ws = workspace();
    let mut client = Client::start_env(Some(&ws_str(ws.path())), &fake_env());
    client.initialize();
    let args = json!({ "proyecto": "SYNTH_Plant", "variante": "VC0" });
    let v = tool_text(&client.call(2, "simular", args));
    assert_eq!(v["success"], json!(true));
    assert_eq!(v["mode"], json!("cli"));
}

#[test]
fn simulation_reports_avisos_pvsyst_from_fake_cli_log() {
    let ws = workspace();
    let temp_local_app_data = tempfile::tempdir().unwrap();
    let mut env = fake_env();
    let local_str = temp_local_app_data.path().to_string_lossy().to_string();
    env.push(("LOCALAPPDATA", &local_str));
    let mut client = Client::start_env(Some(&ws_str(ws.path())), &env);
    client.initialize();
    let args = json!({ "proyecto": "SYNTH_Plant", "variante": "VC0" });
    let v = tool_text(&client.call(2, "simular", args));
    assert_eq!(v["success"], json!(true));
    let avisos = v["avisos_pvsyst"].as_array().expect("avisos_pvsyst");
    assert_eq!(avisos.len(), 4);
    assert_eq!(avisos[0], "ErrorList: 1882 error messages");
    assert_eq!(
        avisos[1],
        "The array Vmpp at 60 °C  is lower than the inverter minimum operating voltage."
    );
    assert_eq!(avisos[2], "This may result in some undervoltage loss.");
    assert_eq!(
        avisos[3],
        "1879 occurrences of MPP voltage lower than min. voltage input of the inverter"
    );
}

#[test]
fn simulation_reports_progress_before_the_response() {
    let ws = workspace();
    let mut env = fake_env();
    env.push(("FAKE_PVSYST_PROGRESO", "1"));
    env.push(("FAKE_PVSYST_SLEEP_MS", "600"));
    let mut client = Client::start_env(Some(&ws_str(ws.path())), &env);
    client.initialize();
    client.send(&json!({
        "jsonrpc": "2.0",
        "id": 2,
        "method": "tools/call",
        "params": {
            "name": "simular",
            "arguments": { "proyecto": "SYNTH_Plant", "variante": "VC0" },
            "_meta": { "progressToken": "t1" }
        }
    }));

    let mut progresos = Vec::new();
    loop {
        let v = client.recv();
        if v["method"] == "notifications/progress" {
            progresos.push(v);
            continue;
        }
        assert_eq!(v["id"], json!(2), "v={v}");
        break;
    }
    assert!(
        progresos
            .iter()
            .any(|v| v["params"]["progressToken"] == json!("t1")),
        "no progress notification: {progresos:?}"
    );
}

#[test]
fn should_answer_ping_while_a_simulation_runs() {
    let ws = workspace();
    let mut env = fake_env();
    env.push(("FAKE_PVSYST_SLEEP_MS", "1500"));
    let mut client = Client::start_env(Some(&ws_str(ws.path())), &env);
    client.initialize();
    client.send(&json!({
        "jsonrpc": "2.0", "id": 2, "method": "tools/call",
        "params": {
            "name": "simular",
            "arguments": { "proyecto": "SYNTH_Plant", "variante": "VC0" }
        }
    }));
    client.send(&json!({ "jsonrpc": "2.0", "id": 3, "method": "ping" }));
    let primero = client.recv();
    assert_eq!(
        primero["id"],
        json!(3),
        "ping was not answered first: {primero}"
    );
    let segundo = client.recv();
    assert_eq!(segundo["id"], json!(2), "{segundo}");
}

#[test]
fn cancelled_simulation_never_answers_and_later_requests_do() {
    let ws = workspace();
    let mut env = fake_env();
    env.push(("FAKE_PVSYST_SLEEP_MS", "5000"));
    let mut client = Client::start_env(Some(&ws_str(ws.path())), &env);
    client.initialize();
    let inicio = std::time::Instant::now();

    client.send(&json!({
        "jsonrpc": "2.0", "id": 7, "method": "tools/call",
        "params": {
            "name": "simular",
            "arguments": { "proyecto": "SYNTH_Plant", "variante": "VC0" }
        }
    }));
    client.send(&json!({
        "jsonrpc": "2.0",
        "method": "notifications/cancelled",
        "params": { "requestId": 7 }
    }));
    client.send(&json!({ "jsonrpc": "2.0", "id": 8, "method": "ping" }));

    let despues_de_cancelar = client.recv();
    assert_eq!(despues_de_cancelar["id"], json!(8), "{despues_de_cancelar}");

    client.send(&json!({ "jsonrpc": "2.0", "id": 9, "method": "ping" }));
    let siguiente = client.recv();
    assert_eq!(siguiente["id"], json!(9), "{siguiente}");
    client.send(&json!({ "jsonrpc": "2.0", "id": 10, "method": "ping" }));
    let cierre = client.recv();
    assert_eq!(cierre["id"], json!(10), "{cierre}");
    assert!(
        inicio.elapsed() < Duration::from_secs(5),
        "cancellation took {:?}",
        inicio.elapsed()
    );
}

#[test]
fn should_truncate_huge_output_and_complete_process() {
    let ws = workspace();
    let mut client = Client::start_env(Some(&ws_str(ws.path())), &fake_env());
    client.initialize();
    let v = tool_text(&client.call(2, "pvsyst_cli", json!({ "comando": "huge-output" })));
    assert_eq!(v["timed_out"], json!(false));
    assert_eq!(v["stdout_truncado"], json!(true));
    assert_eq!(v["exit_code"], json!(0));
}

#[test]
fn should_classify_contradictory_output_as_ambiguous() {
    let ws = workspace();
    let mut client = Client::start_env(Some(&ws_str(ws.path())), &fake_env());
    client.initialize();
    let v = tool_text(&client.call(
        2,
        "pvsyst_cli",
        json!({ "comando": "run-simulation", "opciones": ["--ambiguous-contradictory"] }),
    ));
    assert_eq!(v["success"], json!(false));
    assert_eq!(v["clasificacion"], json!("ambiguo"));
    assert_eq!(v["ambiguo"], json!(true));
}

#[test]
fn should_classify_missing_confirmation_as_ambiguous() {
    let ws = workspace();
    let mut client = Client::start_env(Some(&ws_str(ws.path())), &fake_env());
    client.initialize();
    let v = tool_text(&client.call(
        2,
        "pvsyst_cli",
        json!({ "comando": "run-simulation", "opciones": ["--ambiguous-sin-confirmacion"] }),
    ));
    assert_eq!(v["success"], json!(false));
    assert_eq!(v["clasificacion"], json!("ambiguo"));
    assert_eq!(v["ambiguo"], json!(true));
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
