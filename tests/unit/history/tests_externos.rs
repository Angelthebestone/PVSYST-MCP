//! Snapshots of files outside the workspace (e.g. `PVsyst.ini`, opted-in writes)
//! are restorable only under the same explicit authorisation.

use super::{restaurar, snapshot};
use crate::config::Alcance;
use std::fs;

#[test]
fn should_restore_external_snapshot_only_when_explicitly_permitted() {
    let ws = tempfile::tempdir().expect("ws");
    let fuera = tempfile::tempdir().expect("fuera");
    let ini = fuera.path().join("PVsyst.ini");
    fs::write(&ini, "original").expect("write");
    let id = snapshot(ws.path(), std::slice::from_ref(&ini), "prefs").expect("snapshot");
    fs::write(&ini, "editado").expect("edit");

    let rechazo = restaurar(ws.path(), &id, Alcance::Workspace);
    assert!(
        rechazo.is_err(),
        "sin opt-in no se restaura fuera del workspace"
    );
    assert_eq!(fs::read_to_string(&ini).expect("read"), "editado");

    restaurar(ws.path(), &id, Alcance::Libre).expect("restore with opt-in");
    assert_eq!(fs::read_to_string(&ini).expect("read"), "original");
}
