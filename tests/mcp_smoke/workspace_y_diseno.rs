use super::cliente_mcp::{fixture, synthetic_workspace, tool_text, Client};
use serde_json::json;
use std::path::Path;

fn component_roundtrip(c: &mut Client, tmp: &Path, ws_dir: &Path) {
    let src = fixture("workspace/ComposPV/Inverters/Synth_10kW.OND");
    let src = src.to_string_lossy();
    let outp = tmp.join("exported.OND");
    let r = c.call(
        44,
        "componente",
        json!({
            "accion": "exportar",
            "componente": src, "tipo": "OND", "destino": outp.to_string_lossy(),
            "permitir_fuera_del_workspace": true
        }),
    );
    assert_eq!(r["result"]["isError"], false, "resp={r}");
    assert!(outp.exists());

    let r = c.call(
        45,
        "componente",
        json!({
            "accion": "importar",
            "origen": outp.to_string_lossy(), "tipo": "OND", "nombre_nuevo": "R.OND",
            "permitir_fuera_del_workspace": true
        }),
    );
    assert_eq!(r["result"]["isError"], false, "resp={r}");
    assert!(ws_dir
        .join("ComposPV")
        .join("Inverters")
        .join("R.OND")
        .exists());
}

fn destination_outside_workspace_is_rejected(c: &mut Client, tmp: &Path) {
    let fuera = tmp.join("Param_Modif.dat");
    let r = c.call(
        41,
        "simular",
        json!({
            "accion": "preparar",
            "parametros": { "pi_TimeShiftMax": 40 },
            "destino": fuera.to_string_lossy()
        }),
    );
    assert_eq!(r["result"]["isError"], true, "resp={r}");
    assert!(!fuera.exists());
}

#[test]
fn admin_tools_generate_files_and_roundtrip_component() {
    let tmp = std::env::temp_dir().join(format!("pvsyst-mcp-admin-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&tmp);
    std::fs::create_dir_all(&tmp).unwrap();

    // The client's workspace IS the one we create, so imports land in it.
    let ws_dir = tmp.join("PVsyst8.1_Data");
    let ws_str = ws_dir.to_string_lossy().to_string();
    let mut c = Client::start(Some(ws_str.as_str()));

    // create workspace
    let r = c.call(
        40,
        "workspace",
        json!({ "accion": "crear", "ruta": ws_str }),
    );
    assert_eq!(r["result"]["isError"], false, "resp={r}");
    assert!(ws_dir.join("Projects").is_dir());
    assert!(ws_dir.join("ComposPV").join("Inverters").is_dir());

    destination_outside_workspace_is_rejected(&mut c, &tmp);

    // Param_Modif.dat via simular preparar
    let pm = ws_dir.join("Param_Modif.dat");
    let r = c.call(
        41,
        "simular",
        json!({
            "accion": "preparar",
            "parametros": { "pi_TimeShiftMax": 40 },
            "unidades": { "pi_TimeShiftMax": "min" },
            "destino": pm.to_string_lossy()
        }),
    );
    assert_eq!(r["result"]["isError"], false, "resp={r}");
    let txt = std::fs::read_to_string(&pm).unwrap();
    assert!(txt.starts_with("Hidden parameters: User's modifications;"));
    assert!(txt.contains("pi_TimeShiftMax;0.0000;40;min"));

    // batch file (the base variant must exist; stub files are enough)
    let projects = ws_dir.join("Projects");
    std::fs::create_dir_all(&projects).unwrap();
    std::fs::write(
        projects.join("_DEMO_Residential.PRJ"),
        "PVObject_=pvProject\n",
    )
    .unwrap();
    std::fs::write(
        projects.join("_DEMO_Residential.VC0"),
        "PVObject_=pvVCalcul\n",
    )
    .unwrap();
    let missing = c.call(
        42,
        "simular",
        json!({
            "accion": "preparar",
            "proyecto": "_DEMO_Residential", "variante": "VC7", "filas": [ { "tilt": 25 } ]
        }),
    );
    assert_eq!(missing["result"]["isError"], true, "resp={missing}");
    let bc = ws_dir.join("Batch.csv");
    let r = c.call(43, "simular", json!({
        "accion": "preparar",
        "proyecto": "_DEMO_Residential", "variante": "VC0",
        "filas": [ { "tilt": 25, "azim": 20, "pan": "X.PAN", "n_mod_serie": 15, "n_strings": 2 },
                   { "tilt": 30, "azim": 20, "pan": "X.PAN", "n_mod_serie": 15, "n_strings": 2 } ],
        "variables": ["E_Grid", "PR"],
        "destino": bc.to_string_lossy()
    }));
    assert_eq!(r["result"]["isError"], false, "resp={r}");
    let btxt = std::fs::read_to_string(&bc).unwrap();
    assert!(btxt.contains("SIM_1"));
    assert!(btxt.contains("SIM_2"));
    assert!(btxt.contains("Project;;;_DEMO_Residential.PRJ"));

    component_roundtrip(&mut c, &tmp, &ws_dir);

    // CLI-backed admin: dry runs are instant and hermetic
    let r = c.call(
        46,
        "entorno",
        json!({ "accion": "licencia", "dry_run": true }),
    );
    // The CLI path depends on the machine; only the command itself is hermetic.
    let cmd = tool_text(&r)["command_line"]
        .as_str()
        .unwrap_or("")
        .to_string();
    assert!(cmd.ends_with(" lic-info"), "cmd={cmd}");
    let r = c.call(
        47,
        "entorno",
        json!({ "accion": "exportar_logs", "dry_run": true }),
    );
    assert_eq!(tool_text(&r)["dry_run"], true);

    let _ = std::fs::remove_dir_all(&tmp);
}

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
    let mut c = Client::start(None);
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
