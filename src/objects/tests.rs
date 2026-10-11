use super::*;
use crate::error::Error;
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
fn resuelve_proyecto_con_y_sin_extension() {
    let ws = preparar();
    fs::write(ws.path().join("Projects").join("MCP_T.PRJ"), "x").expect("escribir PRJ");
    let con = resolve_file(ws.path(), &json!({"archivo": "MCP_T.PRJ"})).expect("resolver PRJ");
    let sin = resolve_file(ws.path(), &json!({"archivo": "MCP_T"}))
        .expect("resolver nombre sin extensión");
    assert_eq!(con, ws.path().join("Projects").join("MCP_T.PRJ"));
    assert_eq!(sin, con);
}

#[test]
fn resuelve_variante_sitio_y_meteo_en_su_carpeta() {
    let ws = preparar();
    fs::write(ws.path().join("Projects").join("MCP_T.VC0"), "x").expect("escribir VC0");
    fs::write(ws.path().join("Sites").join("MCP_T.SIT"), "x").expect("escribir SIT");
    fs::write(ws.path().join("Meteo").join("MCP_T.MET"), "x").expect("escribir MET");
    let vc0 = resolve_file(ws.path(), &json!({"archivo": "MCP_T.VC0"})).expect("VC0");
    let sit = resolve_file(ws.path(), &json!({"archivo": "MCP_T.SIT"})).expect("SIT");
    let met = resolve_file(ws.path(), &json!({"archivo": "MCP_T.MET"})).expect("MET");
    assert_eq!(vc0, ws.path().join("Projects").join("MCP_T.VC0"));
    assert_eq!(sit, ws.path().join("Sites").join("MCP_T.SIT"));
    assert_eq!(met, ws.path().join("Meteo").join("MCP_T.MET"));
}

#[test]
fn archivo_inexistente_en_carpeta_pvsyst_es_error() {
    let ws = preparar();
    fs::write(ws.path().join("Projects").join("MCP_T.VC0"), "x").expect("escribir VC0");
    let err = resolve_file(ws.path(), &json!({"archivo": "MCP_X.VC0"}))
        .expect_err("un archivo ausente debe fallar");
    assert!(matches!(err, Error::NotFound(_)), "mensaje: {err}");
    assert!(err.to_string().contains("no encontrado"), "mensaje: {err}");
}

#[test]
fn should_reject_absolute_path_outside_workspace_by_default() {
    let ws = preparar();
    let outside = tempdir().expect("crear outside");
    let file = outside.path().join("ext.PRJ");
    fs::write(&file, "secret").expect("escribir archivo");

    let err = resolve_file(ws.path(), &json!({"archivo": file.to_string_lossy()}))
        .expect_err("debe rechazar ruta absoluta fuera del workspace por defecto");
    assert!(err.to_string().contains("fuera del workspace"));
}

#[test]
fn should_allow_absolute_path_outside_workspace_when_explicitly_permitted() {
    let ws = preparar();
    let outside = tempdir().expect("crear outside");
    let file = outside.path().join("ext.PRJ");
    fs::write(&file, "public").expect("escribir archivo");

    let res = resolve_file(
        ws.path(),
        &json!({
            "archivo": file.to_string_lossy(),
            "permitir_fuera_del_workspace": true,
        }),
    )
    .expect("debe permitir ruta con bandera explicita");
    assert_eq!(res, file);
}

#[test]
fn should_reject_parent_dir_traversal() {
    let ws = preparar();
    let err = resolve_file(ws.path(), &json!({"archivo": "../otro.PRJ"}))
        .expect_err("debe rechazar '..'");
    assert!(err.to_string().contains("contiene '..'"));
}
