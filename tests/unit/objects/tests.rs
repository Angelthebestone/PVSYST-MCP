use super::*;
use crate::error::Error;
use crate::Paths;
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
fn should_list_the_available_objects_when_the_named_one_does_not_exist() {
    let ws = preparar();
    let vc0 = ws.path().join("Projects").join("MCP_T.VC0");
    fs::write(
        &vc0,
        "PVObject_=pvProject
  PVObject_=pvMainArray
    PVObject_=pvSubArray
      NModSerie=10
    End of PVObject pvSubArray
  End of PVObject pvMainArray
End of PVObject pvProject
",
    )
    .expect("escribir VC0");

    let args = json!({
        "workspace": ws.path(),
        "archivo": "MCP_T.VC0",
        "objeto": "pvProjectt"
    });
    let err = leer_objeto(&Paths::resolve(), &args).expect_err("objeto inexistente");
    let msg = err.to_string();
    assert!(msg.contains("pvProjectt"), "{msg}");
    for esperado in ["pvProject", "pvMainArray", "pvSubArray"] {
        assert!(msg.contains(esperado), "lista {esperado}: {msg}");
    }
}

#[test]
fn should_truncate_the_object_list_and_say_how_many_there_are() {
    let ws = preparar();
    let vc0 = ws.path().join("Projects").join("MCP_Muchos.VC0");
    let mut texto = String::from(
        "PVObject_=pvProject
",
    );
    for i in 0..20 {
        texto.push_str(&format!(
            "  PVObject_=pvSubArray[{}]
    NModSerie={i}
  End of PVObject pvSubArray
",
            i + 1
        ));
    }
    texto.push_str(
        "End of PVObject pvProject
",
    );
    fs::write(&vc0, texto).expect("escribir VC0");

    let args = json!({
        "workspace": ws.path(),
        "archivo": "MCP_Muchos.VC0",
        "objeto": "pvNoExiste"
    });
    let err = leer_objeto(&Paths::resolve(), &args).expect_err("objeto inexistente");
    let msg = err.to_string();
    assert!(msg.contains("in total"), "indica cuántos hay: {msg}");
    assert!(!msg.contains("pvSubArray[20]"), "trunca la lista: {msg}");
}

#[test]
fn should_reject_object_values_when_editing_fields() {
    let ws = preparar();
    let vc0 = ws.path().join("Projects").join("MCP_T.VC0");
    let original = "PVObject_=pvProject\n  PVObject_=pvSubArray\n    NStringCh=3\n  End of PVObject pvSubArray\nEnd of PVObject pvProject\n";
    fs::write(&vc0, original).expect("escribir VC0");

    let args = json!({
        "workspace": ws.path(),
        "archivo": "MCP_T.VC0",
        "objeto": "pvSubArray",
        "campos": { "NStringCh": { "anidado": true } }
    });
    let err = editar_objeto(&Paths::resolve(), &args).expect_err("objeto como valor rechazado");
    assert!(err.to_string().contains("NStringCh"), "{err}");
    assert_eq!(fs::read_to_string(&vc0).expect("leer VC0"), original);
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
