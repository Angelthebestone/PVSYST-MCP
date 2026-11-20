use super::*;
use crate::config::Paths;
use serde_json::Value;
use std::path::Path;

/// The fixture variant under `tests/fixtures/workspace`, read, never written.
const VARIANTE: &str = "Projects/SYNTH_Plant.VC0";

/// The objects of the fixture variant in the order the file opens them: the names
/// `editar` takes as `objeto`.
const OBJETOS_DE_LA_VARIANTE: [&str; 6] = [
    "pvVCalcul",
    "pvMeteo",
    "pvSite",
    "pvOrient",
    "pvCircuit",
    "pvSubArray",
];

fn fixture() -> std::path::PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/workspace")
}

fn leer(ws: &Path, args: &Value) -> Value {
    ejecutar_leer(&Paths::para_test(ws), args).expect("leer")
}

#[test]
fn should_not_probe_paths_outside_projects_when_name_is_a_path() {
    let ws = tempfile::tempdir().expect("ws");
    let fuera = tempfile::tempdir().expect("fuera");
    std::fs::create_dir_all(ws.path().join("Projects")).expect("Projects");
    std::fs::write(fuera.path().join("Ajeno.PRJ"), "x").expect("prj");
    let ajeno = fuera.path().join("Ajeno").to_string_lossy().into_owned();

    assert!(!es_proyecto(ws.path(), &ajeno, ""));
    assert!(!es_proyecto(ws.path(), "..\\..\\Ajeno", ""));
}

/// B.4.6: the names come with the file, so editing one field needs no `listar` first.
#[test]
fn should_name_the_objects_it_can_edit_in_the_answer_of_leer() {
    let esperados = json!(OBJETOS_DE_LA_VARIANTE);
    for args in [
        json!({ "archivo": VARIANTE }),
        json!({ "archivo": VARIANTE, "detalle": "completo" }),
        json!({ "archivo": VARIANTE, "objeto": "pvSubArray" }),
    ] {
        let out = leer(&fixture(), &args);
        assert_eq!(out["objetos"], esperados, "args={args}");
    }
}

#[test]
fn should_omit_the_object_names_when_the_file_has_none() {
    let ws = tempfile::tempdir().expect("ws");
    std::fs::create_dir_all(ws.path().join("Projects")).expect("Projects");
    let prj = ws.path().join("Projects").join("SIN_OBJETOS.PRJ");
    std::fs::write(&prj, "Version=8.1.0\r\n").expect("PRJ");

    let out = leer(ws.path(), &json!({ "archivo": "SIN_OBJETOS.PRJ" }));

    assert!(out.get("objetos").is_none(), "{out}");
}
