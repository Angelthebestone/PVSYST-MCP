use super::cliente_mcp::{synthetic_workspace, tool_text, Client};
use serde_json::json;

#[test]
fn pvsyst_cli_dry_run_builds_command() {
    let mut c = Client::start(None);
    let resp = c.call(
        8,
        "pvsyst_cli",
        json!({ "comando": "run-simulation",
                "opciones": ["-w:C:\\ws", "-p:X.PRJ", "-v:VC0", "-rpf:C:\\out.pdf"],
                "dry_run": true }),
    );
    assert_eq!(resp["result"]["isError"], false, "resp={resp}");
    let data = tool_text(&resp);
    let cmd = data["command_line"].as_str().unwrap();
    assert!(cmd.contains("run-simulation"));
    assert!(cmd.contains("-rpf:C:\\out.pdf"));
}

#[test]
fn sitio_crear_dry_run_builds_command() {
    let ws = tempfile::tempdir().expect("ws");
    std::fs::write(ws.path().join("w.csv"), "x").expect("csv");
    let mut c = Client::start(Some(&ws.path().to_string_lossy()));
    let resp = c.call(
        9,
        "sitio",
        json!({ "accion": "crear", "origen": "csv", "nombre_sitio": "S",
                "latitud": 46.2, "longitud": 6.1,
                "weather_data_file": "w.csv", "dry_run": true }),
    );
    assert_eq!(resp["result"]["isError"], false, "resp={resp}");
    let data = tool_text(&resp);
    let cmd = data["command_line"].as_str().unwrap();
    assert!(cmd.contains("create-site"));
    assert!(cmd.contains("-lat:46.2"), "cmd={cmd}");
    assert!(
        cmd.contains(&format!("-wdf:{}", ws.path().join("w.csv").display())),
        "cmd={cmd}"
    );
}

#[test]
fn workspace_historial_and_restore_honor_the_workspace_argument() {
    let base = synthetic_workspace();
    let target = tempfile::tempdir().expect("temp dir");

    let mut c = Client::start(Some(base.path().to_str().unwrap()));
    let mut id = String::new();
    for contenido in ["uno", "dos"] {
        let resp = c.call(
            15,
            "escribir_archivo",
            json!({
                "archivo": "Notas/nota.txt",
                "contenido": contenido,
                "workspace": target.path().to_str().unwrap(),
            }),
        );
        assert_eq!(resp["result"]["isError"], false, "resp={resp}");
        let data = tool_text(&resp);
        if let Some(hist) = data.get("historial_id").and_then(|v| v.as_str()) {
            id = hist.to_string();
        }
    }
    assert!(
        !id.is_empty(),
        "segunda escritura debió snapshotear la primera"
    );

    let resp = c.call(
        16,
        "workspace",
        json!({ "accion": "historial", "workspace": target.path().to_str().unwrap() }),
    );
    assert_eq!(resp["result"]["isError"], false, "resp={resp}");
    let snapshots = tool_text(&resp)["snapshots"]
        .as_array()
        .cloned()
        .unwrap_or_default();
    assert_eq!(
        snapshots.len(),
        1,
        "esperaba 1 snapshot, hubo {snapshots:?}"
    );

    let resp = c.call(
        17,
        "workspace",
        json!({ "accion": "restaurar", "id": id, "workspace": target.path().to_str().unwrap() }),
    );
    assert_eq!(resp["result"]["isError"], false, "resp={resp}");
    let nota = std::fs::read_to_string(target.path().join("Notas").join("nota.txt")).expect("nota");
    assert_eq!(nota, "uno");
}

#[test]
#[ignore = "needs the NASA grids from a PVsyst install: run with --ignored"]
fn sitio_crear_coordenadas_honors_the_workspace_argument() {
    let target = tempfile::tempdir().expect("temp dir");
    let mut c = Client::start(None);
    let resp = c.call(
        17,
        "sitio",
        json!({ "accion": "crear", "origen": "coordenadas",
                "latitud": 46.2, "longitud": 6.1, "nombre_sitio": "MCP_Tmp_Site",
                "dry_run": true, "workspace": target.path().to_str().unwrap() }),
    );
    assert_eq!(resp["result"]["isError"], false, "resp={resp}");
    let csv = target
        .path()
        .join("UserData")
        .join("MCP_Tmp_Site_Monthly.csv");
    assert!(
        csv.exists(),
        "el CSV mensual debe quedar en el workspace pedido"
    );
    assert!(tool_text(&resp)["create_site"]["command_line"]
        .as_str()
        .unwrap()
        .contains("create-site"));
}
