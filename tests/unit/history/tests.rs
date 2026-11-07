//! Behavioral and concurrency tests for the history module.

use super::*;
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

/// Restore confined to the workspace, as every tool call does by default.
pub(super) fn restaurar(ws: &Path, id: &str) -> crate::error::Result<serde_json::Value> {
    super::restaurar(&super::PeticionRestauracion {
        ws,
        id,
        alcance: crate::config::Alcance::Workspace,
        dry_run: false,
    })
}

#[test]
fn should_record_a_snapshot_for_a_creation_and_restore_it_as_absent() {
    let ws = crear_workspace_temporal("reemplazar");
    let f = ws.join("p.txt");
    let creacion = reemplazar(&ws, &f, b"uno", "t").expect("new");
    assert_eq!(fs::read(&f).expect("read"), b"uno");

    let id = reemplazar(&ws, &f, b"dos", "t").expect("replace");
    assert_eq!(fs::read(&f).expect("read"), b"dos");
    restaurar(&ws, &id).expect("restore");
    assert_eq!(fs::read(&f).expect("read"), b"uno");

    restaurar(&ws, &creacion).expect("restore creation");
    assert!(!f.exists(), "restaurar la creación vuelve a 'no existía'");
    let _ = fs::remove_dir_all(&ws);
}
