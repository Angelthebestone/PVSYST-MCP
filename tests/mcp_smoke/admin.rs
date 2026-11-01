//! Administration tools over stdio: workspace creation, batch and parameter files,
//! component transfers and the CLI-backed dry runs.

use super::cliente_mcp::{fixture, tool_text, Client};
use serde_json::json;
use std::path::{Path, PathBuf};

/// A workspace made with `workspace crear` inside a private folder removed at the end.
struct WorkspaceAdmin {
    tmp: tempfile::TempDir,
    ws: PathBuf,
    c: Client,
}

fn workspace_admin() -> WorkspaceAdmin {
    let tmp = tempfile::tempdir().expect("temp dir");
    let ws = tmp.path().join("PVsyst8.1_Data");
    let ws_str = ws.to_string_lossy().to_string();
    let mut c = Client::start(Some(ws_str.as_str()));
    let r = c.call(
        40,
        "workspace",
        json!({ "accion": "crear", "ruta": ws_str }),
    );
    assert_eq!(r["result"]["isError"], false, "resp={r}");
    WorkspaceAdmin { tmp, ws, c }
}

/// The base project a batch needs: stub files are enough.
fn proyecto_base(ws: &Path) {
    let projects = ws.join("Projects");
    std::fs::create_dir_all(&projects).expect("Projects");
    std::fs::write(
        projects.join("_DEMO_Residential.PRJ"),
        "PVObject_=pvProject\n",
    )
    .expect("prj");
    std::fs::write(
        projects.join("_DEMO_Residential.VC0"),
        "PVObject_=pvVCalcul\n",
    )
    .expect("vc0");
}

#[test]
fn should_create_workspace_folders_when_workspace_crear_is_called() {
    let w = workspace_admin();

    assert!(w.ws.join("Projects").is_dir());
    assert!(w.ws.join("ComposPV").join("Inverters").is_dir());
}

#[test]
fn should_reject_param_file_destination_outside_workspace() {
    let mut w = workspace_admin();
    let fuera = w.tmp.path().join("Param_Modif.dat");

    let r = w.c.call(
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
fn should_write_param_modif_when_simular_preparar_gets_parametros() {
    let mut w = workspace_admin();
    let pm = w.ws.join("Param_Modif.dat");

    let r = w.c.call(
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
    let txt = std::fs::read_to_string(&pm).expect("Param_Modif.dat");
    assert!(txt.starts_with("Hidden parameters: User's modifications;"));
    assert!(txt.contains("pi_TimeShiftMax;0.0000;40;min"));
}

#[test]
fn should_reject_batch_when_base_variant_is_missing() {
    let mut w = workspace_admin();
    proyecto_base(&w.ws);

    let r = w.c.call(
        42,
        "simular",
        json!({
            "accion": "preparar",
            "proyecto": "_DEMO_Residential", "variante": "VC7", "filas": [ { "tilt": 25 } ]
        }),
    );

    assert_eq!(r["result"]["isError"], true, "resp={r}");
}

#[test]
fn should_write_batch_csv_when_simular_preparar_gets_filas() {
    let mut w = workspace_admin();
    proyecto_base(&w.ws);
    let bc = w.ws.join("Batch.csv");

    let r = w.c.call(43, "simular", json!({
        "accion": "preparar",
        "proyecto": "_DEMO_Residential", "variante": "VC0",
        "filas": [ { "tilt": 25, "azim": 20, "pan": "X.PAN", "n_mod_serie": 15, "n_strings": 2 },
                   { "tilt": 30, "azim": 20, "pan": "X.PAN", "n_mod_serie": 15, "n_strings": 2 } ],
        "variables": ["E_Grid", "PR"],
        "destino": bc.to_string_lossy()
    }));

    assert_eq!(r["result"]["isError"], false, "resp={r}");
    let btxt = std::fs::read_to_string(&bc).expect("Batch.csv");
    assert!(btxt.contains("SIM_1"));
    assert!(btxt.contains("SIM_2"));
    assert!(btxt.contains("Project;;;_DEMO_Residential.PRJ"));
}

#[test]
fn should_export_and_import_component_when_outside_paths_are_permitted() {
    let mut w = workspace_admin();
    let src = fixture("workspace/ComposPV/Inverters/Synth_10kW.OND");
    let outp = w.tmp.path().join("exported.OND");

    let r = w.c.call(
        44,
        "componente",
        json!({
            "accion": "exportar",
            "componente": src.to_string_lossy(), "tipo": "OND", "destino": outp.to_string_lossy(),
            "permitir_fuera_del_workspace": true
        }),
    );
    assert_eq!(r["result"]["isError"], false, "resp={r}");
    assert!(outp.exists());

    let r = w.c.call(
        45,
        "componente",
        json!({
            "accion": "importar",
            "origen": outp.to_string_lossy(), "tipo": "OND", "nombre_nuevo": "R.OND",
            "permitir_fuera_del_workspace": true
        }),
    );
    assert_eq!(r["result"]["isError"], false, "resp={r}");
    assert!(w
        .ws
        .join("ComposPV")
        .join("Inverters")
        .join("R.OND")
        .exists());
}

#[test]
fn should_build_admin_cli_commands_without_running_them_when_dry_run() {
    let mut w = workspace_admin();

    let r = w.c.call(
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

    let r = w.c.call(
        47,
        "entorno",
        json!({ "accion": "exportar_logs", "dry_run": true }),
    );
    assert_eq!(tool_text(&r)["dry_run"], true);
}
