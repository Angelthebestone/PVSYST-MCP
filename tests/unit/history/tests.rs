//! Behavioral and concurrency tests for the history module.

use super::snapshot::sorted_dirs;
use super::*;
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

/// Restore confined to the workspace, as every tool call does by default.
pub(super) fn restaurar(ws: &Path, id: &str) -> crate::error::Result<serde_json::Value> {
    super::restaurar(&super::PeticionRestauracion {
        ws,
        id,
        alcance: crate::config::Alcance::Workspace,
        dry_run: false,
    })
}

/// Rehearsal of the same restore: nothing on disk may change.
pub(super) fn ensayar(ws: &Path, id: &str) -> crate::error::Result<serde_json::Value> {
    super::restaurar(&super::PeticionRestauracion {
        ws,
        id,
        alcance: crate::config::Alcance::Workspace,
        dry_run: true,
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

#[test]
fn should_not_touch_files_when_the_restore_is_a_rehearsal() {
    let ws = crear_workspace_temporal("dry_run");
    let cambiado = ws.join("cambiado.txt");
    let creado = ws.join("creado.txt");
    fs::write(&cambiado, "antes").expect("write");
    let id = snapshot(&ws, &[cambiado.clone(), creado.clone()], "test").expect("snap");
    fs::write(&cambiado, "despues").expect("write");
    fs::write(&creado, "nuevo").expect("write");

    let ensayo = ensayar(&ws, &id).expect("ensayo");

    assert_eq!(ensayo["simulado"], json!(true));
    assert_eq!(ensayo["cambiarian"], json!(2));
    assert_eq!(fs::read_to_string(&cambiado).expect("read"), "despues");
    assert_eq!(fs::read_to_string(&creado).expect("read"), "nuevo");
    assert_eq!(
        sorted_dirs(&ws.join("UserData").join("mcp_history")).len(),
        1,
        "el ensayo no crea el snapshot previo a la restauración"
    );
    let _ = fs::remove_dir_all(&ws);
}
