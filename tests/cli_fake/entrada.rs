use super::cliente_mcp::{error_text, synthetic_workspace, tool_text, Client};
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

/// `dry_run: null` used to be read as `false`, so a rehearsal ran for real.
#[test]
fn should_refuse_a_dry_run_that_is_null() {
    let ws = workspace();
    let mut client = Client::start_env(Some(&ws_str(ws.path())), &fake_env());
    client.initialize();
    let error = error_text(&client.call(
        2,
        "pvsyst_cli",
        json!({ "comando": "help", "dry_run": null }),
    ));
    assert!(error.contains("dry_run"), "{error}");
}

#[test]
fn should_rehearse_without_running_the_command() {
    let ws = workspace();
    let mut client = Client::start_env(Some(&ws_str(ws.path())), &fake_env());
    client.initialize();
    let v = tool_text(&client.call(
        2,
        "pvsyst_cli",
        json!({ "comando": "run-simulation", "dry_run": true, "opciones": ["-w:C:/ws"] }),
    ));
    assert_eq!(v["dry_run"], json!(true), "{v}");
    assert!(v["command_line"].is_string(), "{v}");
    assert!(
        v.get("success").is_none(),
        "a rehearsal ran the command: {v}"
    );
}

/// The command is the first token of the command line: a blank name or one with
/// spaces inside is a mistake, not something to hand to PVsystCLI.
#[test]
fn should_refuse_a_command_name_with_spaces_or_controls() {
    let ws = workspace();
    let mut client = Client::start_env(Some(&ws_str(ws.path())), &fake_env());
    client.initialize();
    for comando in ["   ", "run simulation", "help\nextra"] {
        let error = error_text(&client.call(2, "pvsyst_cli", json!({ "comando": comando })));
        assert!(error.contains("comando"), "{comando:?} -> {error}");
    }
}

#[test]
fn should_trim_a_command_name_that_is_only_padded() {
    let ws = workspace();
    let mut client = Client::start_env(Some(&ws_str(ws.path())), &fake_env());
    client.initialize();
    let v = tool_text(&client.call(2, "pvsyst_cli", json!({ "comando": " version " })));
    assert_eq!(v["success"], json!(true), "{v}");
    assert!(
        v["command_line"]
            .as_str()
            .unwrap_or_default()
            .ends_with(" version"),
        "{v}"
    );
}
