use super::base64::decode_base64;
use super::*;
use crate::config::Paths;
use serde_json::json;
use std::fs;
use std::path::Path;
use tempfile::tempdir;

fn dummy_paths(ws: &Path) -> Paths {
    Paths::para_test(ws)
}

#[test]
fn should_reject_reads_outside_the_workspace_when_not_explicitly_permitted() {
    let ws = tempdir().expect("ws");
    let outside = tempdir().expect("outside");
    let secret = outside.path().join("secret.txt");
    fs::write(&secret, "confidencial").expect("escribir archivo fuera");
    let paths = dummy_paths(ws.path());

    let absoluta = leer_archivo(
        &paths,
        &json!({ "workspace": ws.path(), "archivo": secret.to_string_lossy() }),
    )
    .expect_err("debe rechazar ruta fuera del workspace");
    assert!(absoluta.to_string().contains("fuera del workspace"));

    let traversal = leer_archivo(
        &paths,
        &json!({ "workspace": ws.path(), "archivo": "../otro.txt" }),
    )
    .expect_err("debe rechazar '..'");
    assert!(traversal.to_string().contains("contiene '..'"));

    let permitida = leer_archivo(
        &paths,
        &json!({
            "workspace": ws.path(),
            "archivo": secret.to_string_lossy(),
            "permitir_fuera_del_workspace": true,
        }),
    )
    .expect("la bandera explícita abre el workspace");
    assert_eq!(permitida["text"], "confidencial");
}

#[test]
fn should_page_large_file_when_window_limits_are_given() {
    let ws = tempdir().expect("ws");
    let file_path = ws.path().join("grande.txt");
    let mut data = String::with_capacity(10_000);
    for i in 0..1000 {
        data.push_str(&format!("{i:09}\n"));
    }
    fs::write(&file_path, &data).expect("escribir archivo grande");

    let paths = dummy_paths(ws.path());
    let p1 = leer_archivo(
        &paths,
        &json!({
            "workspace": ws.path(),
            "archivo": "grande.txt",
            "offset": 0,
            "max_bytes": 1000,
        }),
    )
    .expect("leer página 1");

    assert_eq!(p1["size_bytes"], 10_000);
    assert_eq!(p1["range"], json!([0, 1000]));
    assert_eq!(p1["truncated"], true);
    assert_eq!(p1["next_offset"], 1000);

    let ult = leer_archivo(
        &paths,
        &json!({
            "workspace": ws.path(),
            "archivo": "grande.txt",
            "offset": 9500,
            "max_bytes": 1000,
        }),
    )
    .expect("leer última página");

    assert_eq!(ult["range"], json!([9500, 10_000]));
    assert!(ult.get("truncated").is_none());
    assert!(ult.get("next_offset").is_none());
}

#[test]
fn should_keep_utf8_boundaries_when_truncating_text() {
    let ws = tempdir().expect("ws");
    fs::write(ws.path().join("utf8.txt"), "ñññ").expect("fixture utf8");

    let paths = dummy_paths(ws.path());
    let out = leer_archivo(
        &paths,
        &json!({ "workspace": ws.path(), "archivo": "utf8.txt", "max_bytes": 3 }),
    )
    .expect("read");
    assert_eq!(out["text"], "ñ");
    assert_eq!(out["next_offset"], 2);

    let out2 = leer_archivo(
        &paths,
        &json!({ "workspace": ws.path(), "archivo": "utf8.txt", "offset": 3, "max_bytes": 2 }),
    )
    .expect("read 2");
    assert_eq!(out2["text"], "ñ");
    assert_eq!(out2["range"], json!([2, 4]));
}

#[test]
fn should_serve_binary_window_as_base64_when_text_prefix_hides_binary_bytes() {
    let ws = tempdir().expect("ws");
    let mut datos = "a".repeat(2_000).into_bytes();
    datos.extend_from_slice(b"\x00\xff\x00binario");
    fs::write(ws.path().join("mixto.dat"), &datos).expect("fixture");
    let paths = dummy_paths(ws.path());
    let out = leer_archivo(
        &paths,
        &json!({ "workspace": ws.path(), "archivo": "mixto.dat", "offset": 1_990 }),
    )
    .expect("read");
    assert_eq!(out["is_text"], false);
    assert_eq!(
        decode_base64(out["base64"].as_str().expect("b64")).expect("decode"),
        &datos[1_990..]
    );
}

#[test]
fn should_mark_listing_as_truncated_when_depth_or_count_limit_hit() {
    let ws = tempdir().expect("ws");
    let deep = ws.path().join("d1").join("d2").join("d3").join("d4");
    fs::create_dir_all(&deep).expect("crear arbol profundo");
    fs::write(deep.join("profundo.txt"), "ok").expect("escribir archivo");

    let paths = dummy_paths(ws.path());

    let res_depth = listar_archivos(
        &paths,
        &json!({
            "workspace": ws.path(),
            "carpeta": ".",
            "max_profundidad": 2,
        }),
    )
    .expect("listar con profundidad");
    assert_eq!(res_depth["truncado"], true);

    for i in 0..5 {
        fs::write(ws.path().join(format!("f_{i}.txt")), "x").expect("escribir f");
    }
    let res_count = listar_archivos(
        &paths,
        &json!({
            "workspace": ws.path(),
            "carpeta": ".",
            "max_archivos": 3,
        }),
    )
    .expect("listar con max archivos");
    assert_eq!(res_count["truncado"], true);
    assert!(res_count["count"].as_u64().unwrap_or(0) <= 3);
}

#[test]
fn should_move_file_to_recycle_bin_when_target_name_exists() {
    let ws = tempdir().expect("ws");
    let origen = ws.path().join("prueba.txt");
    fs::write(&origen, "papelera").expect("escribir");

    let movido = mover_a_papelera(ws.path(), &origen).expect("mover a papelera");
    assert!(!origen.exists());
    assert!(movido.exists());
    assert!(movido.starts_with(ws.path().join("UserRecycleBin")));

    fs::write(&origen, "papelera 2").expect("segundo archivo");
    let movido2 = mover_a_papelera(ws.path(), &origen).expect("segundo movimiento");
    assert!(!origen.exists());
    assert!(movido2.exists());
    assert_ne!(movido, movido2);
}
