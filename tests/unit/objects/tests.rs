use super::*;
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
