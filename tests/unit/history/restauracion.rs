//! Byte-exact restore regression for the history module.

use super::tests::{crear_workspace_temporal, restaurar};
use super::*;
use std::fs;

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
