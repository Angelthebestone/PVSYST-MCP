use super::*;
use serde_json::json;
use std::fs;
use tempfile::tempdir;

fn preparar() -> tempfile::TempDir {
    let ws = tempdir().expect("crear carpeta de prueba");
    for carpeta in ["Projects", "Sites", "Meteo"] {
        fs::create_dir_all(ws.path().join(carpeta)).expect("crear carpeta de prueba");
    }
    ws
}

#[test]
fn should_reject_an_absolute_path_outside_the_workspace_by_default() {
    let ws = preparar();
    let outside = tempdir().expect("crear outside");
    let file = outside.path().join("ext.PRJ");
    fs::write(&file, "secret").expect("escribir archivo");

    resolve_file(ws.path(), &json!({"archivo": file.to_string_lossy()}))
        .expect_err("debe rechazar ruta absoluta fuera del workspace por defecto");
}

#[test]
fn should_reject_parent_dir_traversal_when_resolving_a_file() {
    let ws = preparar();
    resolve_file(ws.path(), &json!({"archivo": "../otro.PRJ"})).expect_err("debe rechazar '..'");
}
