use super::cliente_mcp::{fixture, synthetic_workspace, tool_text, Client};
use serde_json::json;
use std::fs;
use std::path::Path;

const FAKE: &str = env!("CARGO_BIN_EXE_fake_pvsystcli");

fn workspace() -> tempfile::TempDir {
    let ws = synthetic_workspace();
    fs::create_dir_all(ws.path().join("Results")).unwrap();
    ws
}

fn fake_env() -> Vec<(&'static str, &'static str)> {
    vec![
        ("PVSYST_CLI", FAKE),
        (
            "FAKE_PVSYST_FIXTURES",
            concat!(env!("CARGO_MANIFEST_DIR"), "/tests/fixtures/cli"),
        ),
    ]
}

fn ws_str(ws: &Path) -> String {
    ws.to_string_lossy().into_owned()
}

#[test]
fn should_snapshot_previous_deliverable_when_regenerated() {
    let ws = workspace();
    let resultados = ws.path().join("Results");
    for nombre in ["MCP_E2E.csv", "MCP_E2E.pdf"] {
        fs::copy(
            fixture(&format!("results/{nombre}")),
            resultados.join(nombre),
        )
        .unwrap();
    }
    let mut client = Client::start_env(Some(&ws_str(ws.path())), &fake_env());
    client.initialize();
    let args = json!({
        "que": "entregable",
        "proyecto": "MCP_E2E",
        "archivo": resultados.join("MCP_E2E.csv").to_string_lossy(),
    });

    let primero = tool_text(&client.call(2, "resultados", args.clone()));
    let segundo = tool_text(&client.call(3, "resultados", args));

    let creacion = primero["historial_id"]
        .as_str()
        .unwrap_or_else(|| panic!("la creación del entregable deja historial: {primero}"));
    let regeneracion = segundo["historial_id"]
        .as_str()
        .unwrap_or_else(|| panic!("la regeneración deja historial: {segundo}"));
    assert_ne!(
        creacion, regeneracion,
        "cada escritura tiene su propio snapshot"
    );
}

/// Variant whose `PNomSizing` (the sizing power PVsyst writes) differs from the
/// array's nominal power the report prints (`Pnom total`): 10 x 2 x 400 W = 8 kWp
/// sized to 4 kWp.
const VARIANTE_DIMENSIONADA_A_MEDIO: &str = "PVObject_=pvVCalcul\r\n  Version=8.1.0\r\n  PVObject_SystemCircuit=pvCircuit\r\n    Version=8.1.0\r\n    PVObject_=pvSubArray\r\n      PVModule=Synth_400W.PAN\r\n      GInverter=Synth_10kW.OND\r\n      NModSerie=10\r\n      NStringCh=2\r\n      NInverter=1\r\n      PNomSizing=4.0\r\n    End of PVObject pvSubArray\r\n  End of PVObject pvCircuit\r\nEnd of PVObject pvVCalcul\r\n";

/// A workspace holding the canned E2E result pair and the project it names.
fn workspace_con_resultado() -> tempfile::TempDir {
    let ws = workspace();
    for nombre in ["MCP_E2E.csv", "MCP_E2E.pdf"] {
        fs::copy(
            fixture(&format!("results/{nombre}")),
            ws.path().join("Results").join(nombre),
        )
        .unwrap();
    }
    fs::write(
        ws.path().join("Projects").join("MCP_E2E.VC0"),
        VARIANTE_DIMENSIONADA_A_MEDIO,
    )
    .unwrap();
    ws
}

#[test]
fn should_take_the_kwp_from_the_report_and_not_from_the_sizing_power() {
    let ws = workspace_con_resultado();
    let mut client = Client::start_env(Some(&ws_str(ws.path())), &fake_env());
    client.initialize();

    // The report prints `Pnom total 9.00 kWp`; `PNomSizing` in the variant is 4.0.
    for (id, que) in [(2, "perdidas"), (3, "mensual"), (4, "entregable")] {
        let out = tool_text(&client.call(
            id,
            "resultados",
            json!({ "que": que, "proyecto": "MCP_E2E" }),
        ));
        assert_eq!(out["kwp"].as_f64(), Some(9.0), "{que}: {out}");
        // 14299 kWh / 9 kWp, the same figure the report's own summary prints.
        assert_eq!(
            out["produccion_especifica_kwh_kwp"].as_f64(),
            Some(1588.8),
            "{que}: {out}"
        );
    }
}
