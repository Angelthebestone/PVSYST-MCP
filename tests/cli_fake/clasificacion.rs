use super::cliente_mcp::{synthetic_workspace, tool_text, Client};
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

/// PVsystCLI prints `Error: Invalid command syntax` and exits 0, so the tool must
/// not report success for a diagnostic command that was rejected.
#[test]
fn should_report_a_rejected_command_as_error() {
    let ws = workspace();
    let mut client = Client::start_env(Some(&ws_str(ws.path())), &fake_env());
    client.initialize();
    for args in [
        json!({ "comando": "help", "opciones": ["--bogus"] }),
        json!({ "comando": "version", "opciones": ["--bogus"] }),
        json!({ "comando": "lic-info", "opciones": ["--bogus"] }),
        json!({ "comando": "help", "opciones": ["help"] }),
    ] {
        let v = tool_text(&client.call(2, "pvsyst_cli", args.clone()));
        assert_eq!(v["success"], json!(false), "{args} -> {v}");
        assert_eq!(v["clasificacion"], json!("error"), "{args} -> {v}");
        assert!(v["detected_error"].is_string(), "{args} -> {v}");
    }
}

/// Command names are case-insensitive, so `VERSION` and `Help` are successes.
#[test]
fn should_accept_a_command_in_any_case() {
    let ws = workspace();
    let mut client = Client::start_env(Some(&ws_str(ws.path())), &fake_env());
    client.initialize();
    for comando in ["VERSION", "Help"] {
        let v = tool_text(&client.call(2, "pvsyst_cli", json!({ "comando": comando })));
        assert_eq!(v["success"], json!(true), "{comando} -> {v}");
        assert_eq!(v["clasificacion"], json!("exito"), "{comando} -> {v}");
    }
}

/// A run whose worker thread died exits 0 and prints only the license banner; the
/// exception is in the Admin log, and the tool must not call that «ambiguo».
#[test]
fn should_report_a_crashed_command_as_error() {
    let ws = workspace();
    let local = tempfile::tempdir().unwrap();
    let mut env = fake_env();
    env.push(("LOCALAPPDATA", local.path().to_str().unwrap()));
    env.push(("FAKE_PVSYST_CRASH", "1"));
    let mut client = Client::start_env(Some(&ws_str(ws.path())), &env);
    client.initialize();
    let v = tool_text(&client.call(2, "pvsyst_cli", json!({ "comando": "run-simulation" })));
    assert_eq!(v["success"], json!(false), "{v}");
    assert_eq!(v["clasificacion"], json!("error"), "{v}");
    assert_eq!(v["crash"], json!("Instruction privilégiée"), "{v}");
}

/// A clear failure message from the CLI is an error, not an ambiguity to retry.
#[test]
fn should_report_a_clear_cli_failure_message_as_error() {
    let ws = workspace();
    let mut client = Client::start_env(Some(&ws_str(ws.path())), &fake_env());
    client.initialize();
    let destino = ws.path().join("no_existe");
    let v = tool_text(&client.call(
        2,
        "pvsyst_cli",
        json!({ "comando": "export-logs", "opciones": [format!("-t:{}", destino.display())] }),
    ));
    assert_eq!(v["success"], json!(false), "{v}");
    assert_eq!(v["clasificacion"], json!("error"), "{v}");
    assert!(
        v["detected_error"]
            .as_str()
            .is_some_and(|e| e.contains("does not exist")),
        "{v}"
    );
}
