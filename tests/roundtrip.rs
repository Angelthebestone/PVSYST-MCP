//! Round-trip checks on real PVsyst project files: an edit must change only the
//! targeted line, and restoring its snapshot must give back the original bytes.
//! The round trip itself needs the PVsyst demo data (--ignored); the hermetic
//! equivalent on a synthetic workspace is in ixture_workspace.rs.
// Test code: failing loudly on unexpected state is the point.
#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::too_many_lines,
    clippy::needless_pass_by_value,
    unreachable_pub
)]

mod cliente_mcp;

use cliente_mcp::{tool_text, Client, DEMO_WS};
use serde_json::json;
use std::fs;
use std::path::{Path, PathBuf};

/// Type of the first `PVObject_<role>=<type>` header in a PVsyst text file.
fn root_type(text: &str) -> Option<String> {
    text.lines()
        .find(|l| l.trim_start_matches('\u{feff}').starts_with("PVObject_"))
        .and_then(|l| l.split_once('=').map(|(_, t)| t.trim().to_string()))
}

/// A private workspace with an empty `Projects` folder, removed when dropped.
fn scratch_workspace() -> tempfile::TempDir {
    let dir = tempfile::tempdir().expect("create temp workspace");
    fs::create_dir_all(dir.path().join("Projects")).expect("create Projects");
    dir
}

#[test]
#[ignore = "needs the PVsyst 8.1.7 demo data: run with --ignored"]
fn editing_one_field_changes_only_that_line_and_restores_exactly() {
    let demo_projects = Path::new(DEMO_WS).join("Projects");
    let mut files: Vec<PathBuf> = fs::read_dir(&demo_projects)
        .expect("read demo projects")
        .filter_map(|e| e.ok().map(|e| e.path()))
        .filter(|p| p.is_file())
        .collect();
    files.sort();

    let mut procesados = 0usize;
    for original in &files {
        let bytes = fs::read(original).expect("read demo file");
        // Demo files may hold cp1252 bytes: decode lossily just to find the root type.
        let text = String::from_utf8_lossy(&bytes).into_owned();
        let Some(tipo) = root_type(&text) else {
            continue;
        };
        procesados += 1;
        let tmp = scratch_workspace();
        let ws = tmp.path();
        let name = original.file_name().expect("file name");
        let copy = ws.join("Projects").join(name);
        fs::copy(original, &copy).expect("copy demo file");
        let before = fs::read(&copy).expect("read copy");

        let mut client = Client::start(Some(ws.to_str().expect("utf-8 path")));
        client.initialize();
        let resp = client.call(
            2,
            "editar",
            json!({
                "archivo": copy.to_string_lossy(),
                "objeto": tipo,
                "campos": { "Comment": "X" }
            }),
        );
        let edit = tool_text(&resp);
        let historial = edit["historial_id"]
            .as_str()
            .expect("editar returns historial_id")
            .to_string();

        let after = fs::read(&copy).expect("read edited");
        let before_lines: Vec<&[u8]> = before.split(|b| *b == b'\n').collect();
        let after_lines: Vec<&[u8]> = after.split(|b| *b == b'\n').collect();
        assert_eq!(
            before_lines.len(),
            after_lines.len(),
            "line count changed for {}",
            name.to_string_lossy()
        );
        let changed = before_lines
            .iter()
            .zip(&after_lines)
            .filter(|(a, b)| a != b)
            .count();
        // Exactly one: zero would mean the edit did not take (no demo has Comment=X).
        assert_eq!(
            changed,
            1,
            "{changed} lines changed for {}",
            name.to_string_lossy()
        );

        let restore = client.call(
            3,
            "workspace",
            json!({ "accion": "restaurar", "id": historial }),
        );
        assert!(tool_text(&restore).is_object(), "restaurar must succeed");
        assert_eq!(
            fs::read(&copy).expect("read restored"),
            before,
            "restored bytes differ for {}",
            name.to_string_lossy()
        );
        drop(client);
    }
    assert!(
        procesados * 2 >= files.len() && procesados > 0,
        "only {procesados} of {} demo files were round-tripped",
        files.len()
    );
}

#[test]
fn escribir_archivo_rejects_traversal_and_writes_nothing_outside() {
    let tmp = scratch_workspace();
    let ws = tmp.path();
    let outside = ws
        .parent()
        .expect("parent")
        .join("evil_mcp_tmp_roundtrip.txt");
    let _ = fs::remove_file(&outside);
    let mut client = Client::start(Some(ws.to_str().expect("utf-8 path")));
    client.initialize();
    let resp = client.call(
        2,
        "escribir_archivo",
        json!({ "archivo": "nonexist/../../evil_mcp_tmp_roundtrip.txt", "contenido": "x" }),
    );
    assert_eq!(
        resp["result"]["isError"], true,
        "traversal must be rejected: {resp}"
    );
    assert!(!outside.exists(), "file escaped the workspace");
}
