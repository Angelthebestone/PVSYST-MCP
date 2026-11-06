//! Behavioural tests for the history module.

use super::snapshot::sorted_dirs;
use serde_json::json;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};

static TEST_COUNTER: AtomicU64 = AtomicU64::new(0);

pub(super) fn crear_workspace_temporal(name: &str) -> PathBuf {
    let n = TEST_COUNTER.fetch_add(1, Ordering::Relaxed);
    let d = std::env::temp_dir().join(format!("mcp_hist_{name}_{}_{n}", std::process::id()));
    let _ = fs::remove_dir_all(&d);
    fs::create_dir_all(&d).expect("tmp dir");
    d
}

pub(super) fn snap_dir(ws: &Path, id: &str) -> PathBuf {
    let d = ws.join("UserData").join("mcp_history").join(id);
    fs::create_dir_all(&d).expect("create snap dir");
    d
}

pub(super) fn escribir_manifiesto(dir: &Path, id: &str, archivos: &serde_json::Value) {
    let m = json!({ "id": id, "fecha": "2026-10-14T10:00:00.000Z", "motivo": "test", "archivos": archivos });
    fs::write(dir.join("manifest.json"), m.to_string()).expect("write manifest");
}

/// Restore confined to the workspace, as every tool call does by default.
pub(super) fn restaurar(ws: &Path, id: &str) -> crate::error::Result<serde_json::Value> {
    super::restaurar(ws, id, crate::config::Alcance::Workspace)
}

#[test]
fn should_reject_restore_and_leave_files_untouched_when_the_original_is_outside_the_workspace() {
    let ws = crear_workspace_temporal("bad_orig");
    let outside_dir = crear_workspace_temporal("outside");
    let outside_file = outside_dir.join("secret.txt");
    fs::write(&outside_file, "intacto").expect("write outside");
    let inside_file = ws.join("local.txt");
    fs::write(&inside_file, "contenido_inicial").expect("write inside");

    let snap_dir = snap_dir(&ws, "snap_outside");
    fs::write(snap_dir.join("000.bin"), "nuevo_contenido").expect("write copia");

    escribir_manifiesto(
        &snap_dir,
        "snap_outside",
        &json!([{ "original": outside_file.to_string_lossy(), "copia": "000.bin", "existia": true }]),
    );

    let resultado = restaurar(&ws, "snap_outside");
    assert!(
        resultado.is_err(),
        "Debe rechazar original fuera del workspace"
    );
    assert_eq!(fs::read_to_string(&outside_file).expect("read"), "intacto");
    assert_eq!(
        fs::read_to_string(&inside_file).expect("read"),
        "contenido_inicial"
    );

    let dirs = sorted_dirs(&ws.join("UserData").join("mcp_history"));
    assert_eq!(
        dirs.len(),
        1,
        "No debe crearse snapshot previo si el manifiesto falla"
    );
    let _ = fs::remove_dir_all(&ws);
    let _ = fs::remove_dir_all(&outside_dir);
}
