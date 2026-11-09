//! `disenar_sistema` end to end against the fake CLI: the site, project, sizing
//! and simulation steps run for real, only PVsystCLI is a double.

use super::cliente_mcp::{synthetic_workspace, tool_text, Client};
use serde_json::{json, Value};
use std::fs;
use std::path::Path;

const FAKE: &str = env!("CARGO_BIN_EXE_fake_pvsystcli");

/// NASA grids for the cell (10, -70), the coordinates of the design below: a
/// monthly climatology is all `create-site` needs from them.
const GRID: &str = "10 -70 5.0 5.2 5.4 5.6 5.8 5.5 5.3 5.1 4.9 4.7 4.8 5.0\n";

/// A `DataRO` with just the three NASA grids, so the site step needs no PVsyst.
fn data_ro() -> tempfile::TempDir {
    let dir = tempfile::tempdir().expect("DataRO");
    for archivo in ["Global_World.txt", "Diffuse_World.TXT", "Tamb10m_World.txt"] {
        fs::write(dir.path().join(archivo), GRID).expect("grid");
    }
    dir
}

/// A fixture folder whose `run-simulation` prints `salida` and writes no report:
/// the shape of a run PVsystCLI could not finish.
fn cli_que_falla(salida: &str) -> tempfile::TempDir {
    let dir = tempfile::tempdir().expect("fixtures CLI");
    for entrada in fs::read_dir(concat!(env!("CARGO_MANIFEST_DIR"), "/tests/fixtures/cli"))
        .expect("fixtures")
        .flatten()
    {
        fs::copy(entrada.path(), dir.path().join(entrada.file_name())).expect("fixture");
    }
    fs::write(dir.path().join("run-simulation.txt"), salida).expect("salida");
    dir
}

fn disenar(client: &mut Client, extra: Value) -> Value {
    let mut args = json!({
        "latitud": 10.0,
        "longitud": -70.0,
        "kwp": 8.0,
        "plantilla": "SYNTH_Plant",
        "nombre": "Diseno",
        "modulo": "Synth_400W.PAN",
        "inversor": "Synth_10kW.OND",
        "n_mod_serie": 10,
        "n_strings": 2,
        "simular": true,
    });
    if let (Some(destino), Some(extra)) = (args.as_object_mut(), extra.as_object()) {
        destino.extend(extra.clone());
    }
    tool_text(&client.call(2, "disenar_sistema", args))
}

fn paso<'a>(out: &'a Value, nombre: &str) -> &'a Value {
    out["pasos"]
        .as_array()
        .into_iter()
        .flatten()
        .find(|p| p["paso"] == nombre)
        .unwrap_or_else(|| panic!("sin paso '{nombre}': {out}"))
}

#[test]
fn should_report_what_pvsyst_printed_when_the_simulation_step_fails() {
    // Given: a CLI whose simulation prints PVsyst's own failure and writes no report
    let ws = synthetic_workspace();
    let data_ro = data_ro();
    let cli = cli_que_falla(
        "Checking license...\nThe sizes of this module are not defined\nSimulation cannot be run.\n",
    );
    let mut client = Client::start_env(
        Some(&ws.path().to_string_lossy()),
        &[
            ("PVSYST_CLI", FAKE),
            ("FAKE_PVSYST_FIXTURES", &cli.path().to_string_lossy()),
            ("PVSYST_DATARO", &data_ro.path().to_string_lossy()),
        ],
    );
    client.initialize();

    // When
    let out = disenar(&mut client, json!({}));

    // Then: the failed step carries PVsyst's words instead of a null detail
    let simulacion = paso(&out, "simulacion");
    assert_eq!(simulacion["ok"], json!(false), "{out}");
    let detalle = simulacion["detail"]
        .as_str()
        .unwrap_or_else(|| panic!("paso de simulación sin detail: {out}"));
    assert!(
        detalle.contains("The sizes of this module are not defined"),
        "detail={detalle}"
    );
    assert!(
        out["errores"].as_array().into_iter().flatten().any(|e| e
            .as_str()
            .is_some_and(|e| e.contains("module are not defined"))),
        "errores={}",
        out["errores"]
    );
}

#[test]
fn should_accept_the_module_name_without_its_extension() {
    // Given: the module named the way `componente` accepts it, without `.PAN`
    let ws = synthetic_workspace();
    let data_ro = data_ro();
    let cli = cli_que_falla("Checking license...\nSimulation done in 0min 2sec\n");
    let mut client = Client::start_env(
        Some(&ws.path().to_string_lossy()),
        &[
            ("PVSYST_CLI", FAKE),
            ("FAKE_PVSYST_FIXTURES", &cli.path().to_string_lossy()),
            ("PVSYST_DATARO", &data_ro.path().to_string_lossy()),
        ],
    );
    client.initialize();

    // When
    let out = disenar(
        &mut client,
        json!({ "modulo": "Synth_400W", "simular": false }),
    );

    // Then: it sizes with that module instead of failing to find it
    assert_eq!(paso(&out, "dimensionado")["ok"], json!(true), "{out}");
    assert_eq!(out["kwp_realizado"].as_f64(), Some(8.0), "{out}");
}

/// The workspace of a design that ran once, for the calls that need one.
#[allow(dead_code)]
fn vc0(ws: &Path) -> String {
    fs::read_to_string(ws.join("Projects").join("Diseno.VC0")).expect("variante")
}
