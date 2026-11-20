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
fn should_reject_empty_contenido_unless_base64_is_explicit() {
    let ws = synthetic_workspace();
    let ws_str = ws.path().to_string_lossy().to_string();
    let mut c = Client::start(Some(&ws_str));
    c.initialize();
    let resp = c.call(
        12,
        "escribir_archivo",
        json!({ "archivo": "vacio.txt", "contenido": "" }),
    );
    let msg = error_text(&resp);
    assert!(msg.contains("must not be empty"), "{msg}");
    assert!(!ws.path().join("vacio.txt").exists());

    let resp = c.call(
        13,
        "escribir_archivo",
        json!({ "archivo": "vacio.txt", "contenido": "", "base64": true }),
    );
    let out = tool_text(&resp);
    assert_eq!(out["bytes_written"], json!(0), "resp={out}");
    assert!(fs::read(ws.path().join("vacio.txt"))
        .expect("archivo")
        .is_empty());
}

#[test]
fn should_reject_a_field_key_containing_equals_without_touching_the_file() {
    let ws = synthetic_workspace();
    let vc0 = ws.path().join("Projects").join("SYNTH_Plant.VC0");
    let antes = fs::read(&vc0).expect("VC0 del fixture");
    let ws_str = ws.path().to_string_lossy().to_string();
    let mut c = Client::start(Some(&ws_str));
    c.initialize();
    let resp = c.call(
        13,
        "editar",
        json!({
            "accion": "campos",
            "archivo": "Projects/SYNTH_Plant.VC0",
            "objeto": "pvSubArray",
            "campos": { "A=B": "1" },
        }),
    );
    let msg = error_text(&resp);
    assert!(msg.contains("A=B"), "{msg}");
    assert_eq!(fs::read(&vc0).expect("VC0 tras el intento"), antes);
}

#[test]
fn should_insert_envejecimiento_when_demo_closes_pvageing_with_named_closer_in_crlf() {
    let ws = synthetic_workspace();
    let vc0 = ws.path().join("Projects").join("SYNTH_Plant.VC0");
    let mut destino = fs::read_to_string(&vc0).expect("VC0 del fixture");
    destino.push_str("PVObject_=pvMainArray\r\nEnd of PVObject pvMainArray\r\n");
    fs::write(&vc0, destino).expect("padre pvMainArray");

    let data_ro = tempfile::tempdir().expect("DataRO temporal");
    let demos = data_ro.path().join("PVsyst8.1_Data").join("Projects");
    fs::create_dir_all(&demos).expect("carpeta de demos");
    let demo = "PVObject_=pvMainArray\r\n  PVObject_Ageing=pvAgeing\r\n    Mode=3\r\n  End of TAgeing\r\nEnd of PVObject pvMainArray\r\n";
    fs::write(demos.join("_DEMO_Commercial.VC8"), demo).expect("demo");

    let ws_str = ws.path().to_string_lossy().to_string();
    let data_str = data_ro.path().to_string_lossy().to_string();
    let mut c = Client::start_env(Some(&ws_str), &[("PVSYST_DATARO", data_str.as_str())]);
    c.initialize();
    let resp = c.call(
        10,
        "proyecto",
        json!({ "accion": "anadir", "proyecto": "SYNTH_Plant", "variante": "VC0", "elemento": "envejecimiento" }),
    );
    let out = tool_text(&resp);
    assert_eq!(
        out["insertados"][0]["tipo"],
        json!("pvAgeing"),
        "resp={out}"
    );
    assert!(fs::read_to_string(&vc0)
        .expect("VC0")
        .contains("PVObject_Ageing=pvAgeing"));
}

#[test]
fn should_not_list_applied_params_in_ignorados() {
    let ws = synthetic_workspace();
    let ws_str = ws.path().to_string_lossy().to_string();
    let mut c = Client::start(Some(&ws_str));
    c.initialize();

    let editado = c.call(
        60,
        "componente",
        json!({
            "accion": "editar", "componente": "Synth_400W.PAN", "tipo": "PAN",
            "campos": { "Model": "Synth 405W" },
        }),
    );
    let out = tool_text(&editado);
    assert_eq!(
        out["changes_applied"][0]["new"],
        json!("Synth 405W"),
        "resp={out}"
    );
    assert!(
        out.get("ignorados").is_none(),
        "'campos' se aplicó, no es ignorado: {out}"
    );

    let escrito = c.call(
        61,
        "escribir_archivo",
        json!({
            "archivo": "UserData/Nota.txt", "contenido": "primera",
            "sobrescribir": true,
        }),
    );
    let out = tool_text(&escrito);
    assert!(out["historial_id"].is_string(), "resp={out}");
    assert!(
        out.get("ignorados").is_none(),
        "'sobrescribir' se consumió, no es ignorado: {out}"
    );
}
