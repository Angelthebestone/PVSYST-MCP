use super::cliente_mcp::{error_text, synthetic_workspace, tool_text, Client};
use serde_json::json;
use std::fs;

#[test]
fn should_delete_only_the_named_variant_when_eliminar_gets_variante() {
    let ws = synthetic_workspace();
    let projects = ws.path().join("Projects");
    let vc0 = fs::read(projects.join("SYNTH_Plant.VC0")).expect("VC0 del fixture");
    fs::write(projects.join("SYNTH_Plant.VC1"), &vc0).expect("crear VC1");
    let results = ws.path().join("Results");
    fs::create_dir_all(&results).expect("carpeta Results");
    fs::write(results.join("SYNTH_Plant_VC1.csv"), "x").expect("csv VC1");
    fs::write(results.join("SYNTH_Plant_VC0.csv"), "x").expect("csv VC0");

    let ws_str = ws.path().to_string_lossy().to_string();
    let mut c = Client::start(Some(&ws_str));
    c.initialize();
    let resp = c.call(
        10,
        "proyecto",
        json!({ "accion": "eliminar", "proyecto": "SYNTH_Plant", "variante": "VC1" }),
    );
    let out = tool_text(&resp);
    assert_eq!(out["count"], json!(2), "resp={out}");

    assert!(projects.join("SYNTH_Plant.PRJ").exists());
    assert!(projects.join("SYNTH_Plant.VC0").exists());
    assert!(!projects.join("SYNTH_Plant.VC1").exists());
    assert!(!results.join("SYNTH_Plant_VC1.csv").exists());
    assert!(results.join("SYNTH_Plant_VC0.csv").exists());
}

#[test]
fn should_reject_eliminar_of_a_variant_that_does_not_exist_without_touching_the_project() {
    let ws = synthetic_workspace();
    let ws_str = ws.path().to_string_lossy().to_string();
    let mut c = Client::start(Some(&ws_str));
    c.initialize();
    let resp = c.call(
        11,
        "proyecto",
        json!({ "accion": "eliminar", "proyecto": "SYNTH_Plant", "variante": "VC9" }),
    );
    let msg = error_text(&resp);
    assert!(msg.contains("VC9"), "{msg}");
    let projects = ws.path().join("Projects");
    assert!(projects.join("SYNTH_Plant.PRJ").exists());
    assert!(projects.join("SYNTH_Plant.VC0").exists());
}

#[test]
fn should_write_an_empty_file_when_escribir_archivo_gets_empty_contenido() {
    let ws = synthetic_workspace();
    let ws_str = ws.path().to_string_lossy().to_string();
    let mut c = Client::start(Some(&ws_str));
    c.initialize();
    let resp = c.call(
        12,
        "escribir_archivo",
        json!({ "archivo": "vacio.txt", "contenido": "" }),
    );
    let out = tool_text(&resp);
    assert_eq!(out["bytes_written"], json!(0), "resp={out}");
    let escrito = fs::read(ws.path().join("vacio.txt")).expect("archivo creado");
    assert!(escrito.is_empty());
}
