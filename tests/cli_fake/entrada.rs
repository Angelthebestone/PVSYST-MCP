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

/// A relative output path must land in the workspace, not in the directory the
/// server was started from.
#[test]
fn should_resolve_a_relative_output_path_against_the_workspace() {
    let ws = workspace();
    let mut client = Client::start_env(Some(&ws_str(ws.path())), &fake_env());
    client.initialize();
    let v = tool_text(&client.call(
        2,
        "pvsyst_cli",
        json!({ "comando": "run-simulation", "opciones": ["-ocf:rel.csv"] }),
    ));
    assert_eq!(v["success"], json!(true), "{v}");
    assert!(ws.path().join("rel.csv").is_file(), "{v}");
}

/// The server promises it never activates or deactivates the licence, so the
/// passthrough refuses those commands and points at the action that returns the
/// command for the user to run with their own key.
#[test]
fn should_refuse_the_commands_that_change_the_licence() {
    let ws = workspace();
    let mut client = Client::start_env(Some(&ws_str(ws.path())), &fake_env());
    client.initialize();
    for (comando, accion) in [
        ("lic-activate", "licencia_activar"),
        ("LIC-DEACTIVATE", "licencia_desactivar"),
        ("lic-sync", "licencia_sync"),
    ] {
        let error = error_text(&client.call(
            2,
            "pvsyst_cli",
            json!({ "comando": comando, "opciones": ["-k:AAAA-BBBB"] }),
        ));
        assert!(error.contains(accion), "{comando} -> {error}");
        assert!(!error.contains("AAAA"), "the key was echoed: {error}");
    }
}

/// Reading the licence state is not a change: `lic-info` keeps working.
#[test]
fn should_keep_reading_the_licence_state() {
    let ws = workspace();
    let mut client = Client::start_env(Some(&ws_str(ws.path())), &fake_env());
    client.initialize();
    let v = tool_text(&client.call(2, "pvsyst_cli", json!({ "comando": "lic-info" })));
    assert_eq!(v["success"], json!(true), "{v}");
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
