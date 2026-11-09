use super::cliente_mcp::{synthetic_workspace, tool_text, Client};
use serde_json::json;
use std::fs;
use std::path::Path;
use std::time::Duration;

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
fn should_kill_the_command_when_it_exceeds_the_timeout() {
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
fn should_answer_later_requests_when_a_simulation_is_cancelled() {
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

/// A cancelled passthrough must kill its child and free the single worker, like
/// `simular` does; otherwise the next request waits for the whole command.
#[test]
fn should_answer_later_requests_when_a_passthrough_is_cancelled() {
    let ws = workspace();
    let mut env = fake_env();
    env.push(("FAKE_PVSYST_SLEEP_MS", "5000"));
    let mut client = Client::start_env(Some(&ws_str(ws.path())), &env);
    client.initialize();
    let inicio = std::time::Instant::now();

    client.send(&json!({
        "jsonrpc": "2.0", "id": 7, "method": "tools/call",
        "params": { "name": "pvsyst_cli", "arguments": { "comando": "help" } }
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
    assert!(
        inicio.elapsed() < Duration::from_secs(4),
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

/// `help run-simulation` prints 65 lines; the answer keeps the last 60 and says so,
/// and `detalle:completo` returns the whole stdout.
#[test]
fn should_flag_a_cut_help_tail_and_return_the_whole_stdout_on_request() {
    let ws = workspace();
    let mut client = Client::start_env(Some(&ws_str(ws.path())), &fake_env());
    client.initialize();
    let args = json!({ "comando": "help", "opciones": ["run-simulation"] });
    let corto = tool_text(&client.call(2, "pvsyst_cli", args.clone()));
    assert_eq!(corto["stdout_tail_truncado"], json!(true), "{corto}");
    assert!(
        !corto["stdout_tail"]
            .as_str()
            .unwrap_or_default()
            .contains("Usage: pvsystcli run-simulation"),
        "{corto}"
    );

    let mut completo = args;
    completo["detalle"] = json!("completo");
    let largo = tool_text(&client.call(3, "pvsyst_cli", completo));
    assert_eq!(largo["stdout_tail_truncado"], json!(false), "{largo}");
    assert!(
        largo["stdout"]
            .as_str()
            .unwrap_or_default()
            .contains("Usage: pvsystcli run-simulation"),
        "{largo}"
    );
}

/// The license banner is noise for a run, but `help` lists `lic-activate`.
#[test]
fn should_keep_the_command_list_of_help() {
    let ws = workspace();
    let mut client = Client::start_env(Some(&ws_str(ws.path())), &fake_env());
    client.initialize();
    let v = tool_text(&client.call(2, "pvsyst_cli", json!({ "comando": "help" })));
    assert!(
        v["stdout_tail"]
            .as_str()
            .unwrap_or_default()
            .contains("lic-activate"),
        "{v}"
    );
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

/// Runs one simulation with the fake CLI and the given extra environment.
fn simular_con(env: &[(&str, &str)]) -> serde_json::Value {
    let ws = workspace();
    let local = tempfile::tempdir().expect("LOCALAPPDATA temporal");
    let mut completo = fake_env();
    completo.push(("LOCALAPPDATA", local.path().to_str().expect("utf8")));
    completo.extend_from_slice(env);
    let mut client = Client::start_env(Some(&ws_str(ws.path())), &completo);
    client.initialize();
    tool_text(&client.call(
        2,
        "simular",
        json!({ "proyecto": "SYNTH_Plant", "variante": "VC0" }),
    ))
}

#[test]
fn should_not_report_success_when_pvsyst_flags_the_result_as_incoherent() {
    // Given: a run whose log carries PVsyst's incoherence warning
    let v = simular_con(&[("FAKE_PVSYST_AVISO_INCOHERENTE", "1")]);

    // Then: the result is a failure that names PVsyst's warning, not a success
    assert_eq!(v["success"], json!(false), "v={v}");
    assert_eq!(v["clasificacion"], json!("fallo"), "v={v}");
    assert!(
        v["coherencia"]["motivos"][0]
            .as_str()
            .is_some_and(|m| m.contains("Incoherencia en los resultados")),
        "v={v}"
    );
}

#[test]
fn should_not_report_success_when_the_report_pr_is_physically_impossible() {
    // Given: a run whose report says PR 130.6 %
    let informe = tempfile::tempdir().expect("informe");
    let pdf = informe.path().join("imposible.pdf");
    super::cliente_mcp::escribir_informe_con_pr(&pdf, 130.6);
    let v = simular_con(&[("FAKE_PVSYST_PDF", pdf.to_str().expect("utf8"))]);

    // Then: the impossible PR is a failure, not a success with a nice number
    assert_eq!(v["success"], json!(false), "v={v}");
    assert_eq!(v["clasificacion"], json!("fallo"), "v={v}");
    assert!(
        v["coherencia"]["motivos"][0]
            .as_str()
            .is_some_and(|m| m.contains("130.6")),
        "v={v}"
    );
}
