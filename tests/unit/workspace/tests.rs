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
fn should_reject_path_outside_workspace_by_default() {
    let ws = tempdir().expect("ws");
    let outside = tempdir().expect("outside");
    let secret = outside.path().join("secret.txt");
    fs::write(&secret, "confidencial").expect("escribir archivo fuera");

    let paths = dummy_paths(ws.path());
    let args = json!({
        "workspace": ws.path(),
        "archivo": secret.to_string_lossy(),
    });
    let err = leer_archivo(&paths, &args).expect_err("debe rechazar ruta fuera del workspace");
    assert!(err.to_string().contains("fuera del workspace"));
}

#[test]
fn should_suggest_the_pvsyst_file_when_leer_gets_a_bare_site_name() {
    let ws = tempdir().expect("ws");
    fs::create_dir_all(ws.path().join("Sites")).expect("sites dir");
    fs::write(ws.path().join("Sites/E2E_Site.SIT"), "x").expect("sitio");
    let paths = dummy_paths(ws.path());
    let args = json!({ "workspace": ws.path(), "archivo": "E2E_Site" });
    let err = leer_archivo(&paths, &args).expect_err("un nombre suelto no es un archivo");
    let texto = err.to_string();
    assert!(
        texto.contains("Archivo 'E2E_Site' no encontrado"),
        "{texto}"
    );
    assert!(texto.contains("Sites/E2E_Site.SIT"), "{texto}");
}

#[test]
fn should_allow_path_outside_workspace_when_explicitly_permitted() {
    let ws = tempdir().expect("ws");
    let outside = tempdir().expect("outside");
    let secret = outside.path().join("secret.txt");
    fs::write(&secret, "contenido publico").expect("escribir archivo fuera");

    let paths = dummy_paths(ws.path());
    let args = json!({
        "workspace": ws.path(),
        "archivo": secret.to_string_lossy(),
        "permitir_fuera_del_workspace": true,
    });
    let res = leer_archivo(&paths, &args).expect("debe permitir ruta con bandera explicita");
    assert_eq!(res["text"], "contenido publico");
}

#[test]
fn should_reject_parent_dir_traversal() {
    let ws = tempdir().expect("ws");
    let paths = dummy_paths(ws.path());
    let args = json!({
        "workspace": ws.path(),
        "archivo": "../otro.txt",
    });
    let err = leer_archivo(&paths, &args).expect_err("debe rechazar '..'");
    assert!(err.to_string().contains("contiene '..'"));
}

#[test]
fn should_page_large_file_with_correct_windows_and_next_offset() {
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

    let p2 = leer_archivo(
        &paths,
        &json!({
            "workspace": ws.path(),
            "archivo": "grande.txt",
            "offset": 1000,
            "max_bytes": 1000,
        }),
    )
    .expect("leer página 2");

    assert_eq!(p2["range"], json!([1000, 2000]));
    assert_eq!(p2["truncated"], true);
    assert_eq!(p2["next_offset"], 2000);

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
fn should_respect_utf8_char_boundaries_on_truncation() {
    let ws = tempdir().expect("ws");
    fs::write(ws.path().join("utf8.txt"), "ñññ").expect("fixture utf8");

    let paths = dummy_paths(ws.path());
    let args = json!({ "workspace": ws.path(), "archivo": "utf8.txt", "max_bytes": 3 });
    let out = leer_archivo(&paths, &args).expect("read");
    assert_eq!(out["text"], "ñ");
    assert_eq!(out["next_offset"], 2);

    let args2 =
        json!({ "workspace": ws.path(), "archivo": "utf8.txt", "offset": 3, "max_bytes": 2 });
    let out2 = leer_archivo(&paths, &args2).expect("read 2");
    assert_eq!(out2["text"], "ñ");
    assert_eq!(out2["range"], json!([2, 4]));
}

#[test]
fn should_serve_window_as_base64_when_text_prefix_hides_binary_bytes() {
    let ws = tempdir().expect("ws");
    let mut datos = "a".repeat(2_000).into_bytes();
    datos.extend_from_slice(b"\x00\xff\x00binario");
    fs::write(ws.path().join("mixto.dat"), &datos).expect("fixture");
    let paths = dummy_paths(ws.path());
    let args = json!({ "workspace": ws.path(), "archivo": "mixto.dat", "offset": 1_990 });
    let out = leer_archivo(&paths, &args).expect("read");
    assert_eq!(out["is_text"], false);
    assert_eq!(
        decode_base64(out["base64"].as_str().expect("b64")).expect("decode"),
        &datos[1_990..]
    );
}

#[test]
#[ignore = "requiere privilegios de SO para crear enlaces simbólicos"]
fn should_terminate_recursion_on_symlink_or_junction_cycle() {
    let ws = tempdir().expect("ws");
    let dir_a = ws.path().join("dir_a");
    let dir_b = dir_a.join("dir_b");
    fs::create_dir_all(&dir_b).expect("crear subdirectorios");
    fs::write(dir_b.join("archivo.txt"), "prueba").expect("escribir archivo");

    #[cfg(windows)]
    let symlink_res = std::os::windows::fs::symlink_dir(&dir_a, dir_b.join("ciclo"));
    #[cfg(unix)]
    let symlink_res = std::os::unix::fs::symlink(&dir_a, dir_b.join("ciclo"));

    symlink_res.expect("Creación de symlink");

    let paths = dummy_paths(ws.path());
    let res = listar_archivos(
        &paths,
        &json!({
            "workspace": ws.path(),
            "carpeta": "dir_a",
            "recursivo": true,
        }),
    )
    .expect("listar no debe ciclar");

    assert!(res["count"].as_u64().unwrap_or(0) >= 1);
}

#[test]
fn should_not_panic_on_underflow_in_leer_ventana_texto() {
    let ws = tempdir().expect("ws");
    let mut datos = vec![b'a'; 1024];
    datos.extend_from_slice(&[0x80; 4]);
    datos.push(b'B');
    fs::write(ws.path().join("underflow.txt"), &datos).expect("escribir archivo");

    let paths = dummy_paths(ws.path());
    let res = leer_archivo(
        &paths,
        &json!({
            "workspace": ws.path(),
            "archivo": "underflow.txt",
            "offset": 1027,
            "max_bytes": 10,
        }),
    );
    assert!(
        res.is_ok(),
        "No debe entrar en pánico por underflow: {res:?}"
    );
}

#[test]
fn should_correctly_snap_four_byte_emoji_boundary() {
    let ws = tempdir().expect("ws");
    let mut datos = "prefix_".as_bytes().to_vec();
    datos.extend_from_slice("😀suffix".as_bytes());
    fs::write(ws.path().join("emoji.txt"), &datos).expect("escribir emoji");

    let paths = dummy_paths(ws.path());
    let res = leer_archivo(
        &paths,
        &json!({
            "workspace": ws.path(),
            "archivo": "emoji.txt",
            "offset": 8,
            "max_bytes": 10,
        }),
    )
    .expect("lectura exitosa");
    assert!(res.get("text").is_some());
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
fn should_reject_invalid_filenames_and_reserved_names() {
    assert!(validar_nombre("").is_err());
    assert!(validar_nombre("con").is_err());
    assert!(validar_nombre("AUX.txt").is_err());
    assert!(validar_nombre("a/b").is_err());
    assert!(validar_nombre("..").is_err());
    assert!(validar_nombre("valido_123.txt").is_ok());
}

#[test]
fn should_move_file_to_recycle_bin_and_disambiguate_name() {
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
