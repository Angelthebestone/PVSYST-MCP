//! Byte-exact restore regression for the history module.

use super::tests::{crear_workspace_temporal, restaurar};
use super::*;
use std::fs;

#[test]
fn should_restore_files_and_allow_undo_via_previous_snapshot() {
    let ws = crear_workspace_temporal("restore_undo");
    let kept = ws.join("kept.txt");
    let created = ws.join("created.txt");
    fs::write(&kept, "antes").expect("write");
    let id = snapshot(&ws, &[kept.clone(), created.clone()], "test").expect("snap");
    fs::write(&kept, "despues").expect("write");
    fs::write(&created, "nuevo").expect("write");

    let res = restaurar(&ws, &id).expect("restore");
    assert_eq!(fs::read_to_string(&kept).expect("read"), "antes");
    assert!(!created.exists());

    let previo = res["historial_id_previo"].as_str().expect("previo");
    restaurar(&ws, previo).expect("undo restore");
    assert_eq!(fs::read_to_string(&kept).expect("read"), "despues");
    assert_eq!(fs::read_to_string(&created).expect("read"), "nuevo");
    let _ = fs::remove_dir_all(&ws);
}

#[test]
fn should_reject_path_like_snapshot_ids() {
    let ws = crear_workspace_temporal("badid");
    assert!(restaurar(&ws, "../x").is_err());
    assert!(restaurar(&ws, "a/b").is_err());
    assert!(restaurar(&ws, "a\\b").is_err());
    assert!(restaurar(&ws, "").is_err());
    let _ = fs::remove_dir_all(&ws);
}

#[test]
fn should_restore_bytes_identical_to_the_snapshot() {
    let ws = crear_workspace_temporal("bytes_identicos");
    let f = ws.join("v.bin");
    let original = b"PVObject_=pvRoot\r\n  A=1\r\nEnd of PVObject pvRoot\r\n\x00\xff";
    fs::write(&f, original).expect("write");
    let id = snapshot(&ws, std::slice::from_ref(&f), "bytes").expect("snap");
    fs::write(&f, b"otros bytes").expect("write other");
    restaurar(&ws, &id).expect("restore");
    assert_eq!(fs::read(&f).expect("read"), original);
    let _ = fs::remove_dir_all(&ws);
}
