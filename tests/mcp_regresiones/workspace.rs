use super::cliente_mcp::{synthetic_workspace, tool_text, Client};
use serde_json::{json, Value};
use std::fs;

fn error_text(resp: &Value) -> String {
    assert_eq!(resp["result"]["isError"], json!(true), "resp={resp}");
    resp["result"]["content"][0]["text"]
        .as_str()
        .unwrap_or_else(|| panic!("sin texto en {resp}"))
        .to_string()
}

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
fn should_reject_met_and_sit_files_above_the_read_limit() {
    let ws = synthetic_workspace();
    let ws_str = ws.path().to_string_lossy().to_string();
    let grande = 64 * 1024 * 1024 + 1;
    for ruta in [
        ws.path().join("Meteo").join("Grande.MET"),
        ws.path().join("Sites").join("Grande.SIT"),
    ] {
        fs::create_dir_all(ruta.parent().expect("carpeta")).expect("crear carpeta");
        fs::File::create(&ruta)
            .expect("crear archivo")
            .set_len(grande)
            .expect("tamaño");
    }

    let mut c = Client::start(None);
    c.initialize();
    for (i, archivo) in ["Meteo/Grande.MET", "Sites/Grande.SIT"].iter().enumerate() {
        let resp = c.call(
            70 + i as u64,
            "leer",
            json!({ "archivo": archivo, "workspace": ws_str }),
        );
        let msg = error_text(&resp);
        assert!(msg.contains("máximo"), "{archivo}: {msg}");
    }
}

#[test]
fn should_revert_applied_entries_when_a_later_restore_entry_fails() {
    let ws = synthetic_workspace();
    let ws_str = ws.path().to_string_lossy().to_string();
    let mut c = Client::start(None);
    c.initialize();

    let crear = |c: &mut Client, id: u64| {
        c.call(
            id,
            "proyecto",
            json!({
                "accion": "crear", "plantilla": "SYNTH_Plant", "nombre_nuevo": "Roll",
                "sobrescribir": true, "workspace": ws_str,
            }),
        )
    };
    let primera = crear(&mut c, 80);
    assert_eq!(primera["result"]["isError"], json!(false), "resp={primera}");
    let segunda = crear(&mut c, 81);
    let id = tool_text(&segunda)["historial_id"]
        .as_str()
        .expect("historial_id de la segunda creación")
        .to_string();

    let prj = ws.path().join("Projects").join("Roll.PRJ");
    let resp = c.call(
        82,
        "escribir_archivo",
        json!({
            "archivo": "Projects/Roll.PRJ",
            "contenido": "PVObject_=pvProject\n  Comment=modificado antes de restaurar\nEnd of PVObject pvProject\n",
            "workspace": ws_str,
        }),
    );
    assert_eq!(resp["result"]["isError"], json!(false), "resp={resp}");
    let previo = fs::read(&prj).expect("PRJ previo");

    // El original del segundo archivo pasa a ser una carpeta: la restauración
    // fallará tras haber restaurado el primero.
    let var = ws.path().join("Projects").join("Roll.VC0");
    fs::remove_file(&var).expect("quitar VC0");
    fs::create_dir(&var).expect("VC0 como carpeta");

    let resp = c.call(
        83,
        "workspace",
        json!({ "accion": "restaurar", "id": id, "workspace": ws_str }),
    );
    let msg = error_text(&resp);
    assert!(msg.contains("se revirtieron"), "{msg}");
    assert_eq!(
        fs::read(&prj).expect("PRJ tras el rollback"),
        previo,
        "el archivo ya restaurado volvió a su estado previo"
    );
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

#[test]
fn should_restore_replaced_files_when_import_publication_fails() {
    let ws = synthetic_workspace();
    let ws_str = ws.path().to_string_lossy().to_string();
    let mut c = Client::start(None);
    c.initialize();

    let origen = c.call(
        87,
        "workspace",
        json!({ "accion": "exportar", "destino": "origen.zip", "workspace": ws_str }),
    );
    assert_eq!(origen["result"]["isError"], json!(false), "resp={origen}");

    // 'PVModules' es un archivo: tras publicar ComposPV/Inverters/... la
    // publicación de ComposPV/PVModules/... no puede crear su carpeta.
    let destino = ws.path().join("destino");
    fs::create_dir_all(destino.join("ComposPV")).expect("destino");
    fs::write(destino.join("ComposPV").join("PVModules"), "bloqueo").expect("bloqueo");

    let resp = c.call(
        88,
        "workspace",
        json!({
            "accion": "importar", "zip": "origen.zip", "destino": "destino",
            "sobrescribir": true, "workspace": ws_str,
        }),
    );
    let msg = error_text(&resp);
    assert!(msg.contains("se restauraron"), "{msg}");
    assert!(
        !destino
            .join("ComposPV")
            .join("Inverters")
            .join("Synth_10kW.OND")
            .exists(),
        "lo publicado antes del fallo se revirtió"
    );
    assert_eq!(
        fs::read_to_string(destino.join("ComposPV").join("PVModules")).expect("bloqueo"),
        "bloqueo"
    );
}

#[test]
fn should_not_pollute_default_workspace_when_creating_site_with_explicit_workspace() {
    let default_ws = tempfile::tempdir().expect("default ws");
    let target_ws = tempfile::tempdir().expect("target ws");
    let mut paths = pvsyst_mcp::Paths::resolve();
    paths.workspace = default_ws.path().to_path_buf();

    let args = json!({
        "accion": "crear",
        "origen": "coordenadas",
        "latitud": 46.2,
        "longitud": 6.1,
        "nombre_sitio": "MCP_Isolated_Site",
        "dry_run": true,
        "workspace": target_ws.path().to_str().unwrap(),
    });

    let resp = pvsyst_mcp::Server::new(paths)
        .handle_line(
            &json!({
                "jsonrpc": "2.0",
                "id": 1,
                "method": "tools/call",
                "params": {
                    "name": "sitio",
                    "arguments": args
                }
            })
            .to_string(),
        )
        .expect("handle line");
    let v: Value = serde_json::from_str(&resp).expect("valid json");
    assert_eq!(v["result"]["isError"], false, "resp={v}");

    assert!(target_ws
        .path()
        .join("UserData")
        .join("MCP_Isolated_Site_Monthly.csv")
        .exists());
    assert!(!default_ws.path().join("UserData").exists());
}
