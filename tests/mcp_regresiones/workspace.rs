use super::cliente_mcp::{error_text, synthetic_workspace, Client};
use serde_json::json;
use std::fs;

#[test]
fn should_reject_a_discordant_closing_type() {
    let ws = synthetic_workspace();
    let ws_str = ws.path().to_string_lossy().to_string();
    let mut c = Client::start(Some(&ws_str));
    c.initialize();

    let escrito = c.call(
        40,
        "escribir_archivo",
        json!({
            "archivo": "Proyectos/Discordante.PRJ",
            "contenido": "PVObject_=pvSite\n  Site=Geneva\nEnd of PVObject pvProject\n",
        }),
    );
    assert_eq!(escrito["result"]["isError"], json!(false), "resp={escrito}");

    let resp = c.call(
        41,
        "editar",
        json!({
            "accion": "campos",
            "archivo": "Proyectos/Discordante.PRJ",
            "objeto": "pvSite",
            "campos": { "Site": "Otro" },
        }),
    );
    let msg = error_text(&resp);
    assert!(msg.contains("discordante"), "{msg}");
}

#[test]
fn should_import_workspace_zip_replacing_existing_files() {
    let ws = synthetic_workspace();
    let ws_str = ws.path().to_string_lossy().to_string();
    let mut c = Client::start(None);
    c.initialize();

    let origen = c.call(
        85,
        "workspace",
        json!({ "accion": "exportar", "destino": "origen.zip", "workspace": ws_str }),
    );
    assert_eq!(origen["result"]["isError"], json!(false), "resp={origen}");

    let destino = ws.path().join("destino");
    fs::create_dir_all(destino.join("Projects")).expect("destino");
    fs::write(destino.join("Projects").join("SYNTH_Plant.PRJ"), "viejo").expect("previo");

    let resp = c.call(
        86,
        "workspace",
        json!({
            "accion": "importar", "zip": "origen.zip", "destino": "destino",
            "sobrescribir": true, "workspace": ws_str,
        }),
    );
    assert_eq!(resp["result"]["isError"], json!(false), "resp={resp}");
    assert_eq!(
        fs::read(destino.join("Projects").join("SYNTH_Plant.PRJ")).expect("PRJ"),
        fs::read(ws.path().join("Projects").join("SYNTH_Plant.PRJ")).expect("fuente"),
        "el archivo reemplazado quedó con el contenido del zip"
    );
    assert!(destino
        .join("ComposPV")
        .join("PVModules")
        .join("Synth_400W.PAN")
        .is_file());
    let restos: Vec<String> = fs::read_dir(ws.path())
        .expect("ws")
        .flatten()
        .map(|e| e.file_name().to_string_lossy().to_string())
        .filter(|n| n.contains(".mcp_import_"))
        .collect();
    assert!(
        restos.is_empty(),
        "sin temporales de importación: {restos:?}"
    );
}
