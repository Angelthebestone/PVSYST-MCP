use super::escribir_archivo;
use crate::config::Paths;
use serde_json::json;
use std::fs;
use std::path::Path;
use tempfile::tempdir;

fn dummy_paths(ws: &Path) -> Paths {
    Paths::para_test(ws)
}

#[test]
fn should_reject_empty_content_unless_base64_is_explicit() {
    let ws = tempdir().expect("ws");
    let paths = dummy_paths(ws.path());

    let err = escribir_archivo(
        &paths,
        &json!({ "archivo": "UserData/Vacio.txt", "contenido": "" }),
    )
    .expect_err("contenido vacío rechazado");
    assert!(
        err.to_string().contains("'contenido' must not be empty"),
        "{err}"
    );
    assert!(!ws.path().join("UserData").join("Vacio.txt").exists());

    let out = escribir_archivo(
        &paths,
        &json!({ "archivo": "UserData/Vacio.txt", "contenido": "", "base64": true }),
    )
    .expect("base64 explícito sí puede escribir 0 bytes");
    assert_eq!(out["bytes_written"], json!(0));
}

#[test]
fn should_refuse_overwriting_when_sobrescribir_is_explicitly_false() {
    let ws = tempdir().expect("ws");
    let paths = dummy_paths(ws.path());
    let destino = ws.path().join("UserData").join("Nota.txt");
    fs::create_dir_all(destino.parent().expect("carpeta")).expect("carpeta");
    fs::write(&destino, "original").expect("previo");

    let err = escribir_archivo(
        &paths,
        &json!({ "archivo": "UserData/Nota.txt", "contenido": "nuevo", "sobrescribir": false }),
    )
    .expect_err("sobrescribir:false veta el reemplazo");
    assert!(err.to_string().contains("already exists"), "{err}");
    assert_eq!(fs::read_to_string(&destino).expect("leer"), "original");

    escribir_archivo(
        &paths,
        &json!({ "archivo": "UserData/Nota.txt", "contenido": "nuevo" }),
    )
    .expect("sin el veto se sobrescribe como siempre");
    assert_eq!(fs::read_to_string(&destino).expect("leer"), "nuevo");
}

#[test]
fn should_record_history_when_creating_a_new_file() {
    let ws = tempdir().expect("ws");
    let paths = dummy_paths(ws.path());
    let args = json!({ "archivo": "UserData/Nota.txt", "contenido": "primera version" });

    let out = escribir_archivo(&paths, &args).expect("creación");

    let id = out["historial_id"]
        .as_str()
        .expect("la creación también deja historial_id");
    let historial = crate::history::listar(ws.path(), 20);
    assert_eq!(historial["total"], json!(1), "historial={historial}");
    assert_eq!(historial["snapshots"][0]["id"], json!(id));
}
