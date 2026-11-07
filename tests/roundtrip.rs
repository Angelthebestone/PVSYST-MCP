//! Workspace confinement on real writes: a traversal must be rejected and leave
//! nothing outside the workspace. The byte-exact edit/restore round trip is
//! covered hermetically in `fixture_workspace.rs`.
// Test code: failing loudly on unexpected state is the point.
#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::needless_pass_by_value,
    unreachable_pub
)]

mod cliente_mcp;

use cliente_mcp::Client;
use serde_json::json;
use std::fs;

/// A private workspace with an empty `Projects` folder, removed when dropped.
fn scratch_workspace() -> tempfile::TempDir {
    let dir = tempfile::tempdir().expect("create temp workspace");
    fs::create_dir_all(dir.path().join("Projects")).expect("create Projects");
    dir
}

#[test]
fn should_reject_traversal_and_write_nothing_outside_when_writing_a_file() {
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
