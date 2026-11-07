//! Hermetic end-to-end tests: the server runs against the synthetic workspace in
//! `tests/fixtures/workspace`, so they need no PVsyst installation, network or clock.
// Test code: failing loudly on unexpected state is the point.
#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::needless_pass_by_value,
    unreachable_pub
)]

mod cliente_mcp;

use cliente_mcp::{error_text, synthetic_workspace, tool_text, Client};
use serde_json::json;
use std::fs;

fn start(ws: &tempfile::TempDir) -> Client {
    let mut c = Client::start(Some(ws.path().to_str().expect("utf-8 path")));
    c.initialize();
    c
}

/// A grid variant with the `pvSystem` that storage is configured on.
const VARIANTE_CON_SISTEMA: &str = "PVObject_=pvVCalcul\r\n  Version=8.1.0\r\n  PVObject_SystemCircuit=pvCircuit\r\n    Version=8.1.0\r\n    PVObject_=pvSubArray\r\n      PVModule=Synth_400W.PAN\r\n      GInverter=Synth_10kW.OND\r\n    End of PVObject pvSubArray\r\n  End of PVObject pvCircuit\r\n  PVObject_System=pvSystem\r\n    Version=8.1.0\r\n    SystemType=Grid\r\n  End of PVObject pvSystem\r\nEnd of PVObject pvVCalcul\r\n";

/// The catalogue's demo variant: storage plus the load it used to bring along.
const DEMO_CON_CARGA: &str = "PVObject_=pvVCalcul\r\n  Version=8.1.0\r\n  PVObject_UserNeeds=pvLoad\r\n    LoadType=DailyProfile\r\n  End of PVObject pvLoad\r\n  PVObject_GridStorage=pvGridStorage\r\n    Flags=$00\r\n    Strategy=SelfConsumption\r\n  End of PVObject pvGridStorage\r\nEnd of PVObject pvVCalcul\r\n";

#[test]
fn should_refuse_a_missing_battery_and_add_no_load_when_configuring_storage() {
    let ws = synthetic_workspace();
    let vc0 = ws.path().join("Projects").join("SYNTH_Plant.VC0");
    fs::write(&vc0, VARIANTE_CON_SISTEMA).expect("variant with pvSystem");
    let baterias = ws.path().join("ComposPV").join("Batteries");
    fs::create_dir_all(&baterias).expect("ComposPV/Batteries");
    fs::write(
        baterias.join("Generic_LI_Module_180Ah.BTR"),
        "PVObject_=pvBattery\r\nEnd of PVObject pvBattery\r\n",
    )
    .expect("battery");
    let data_ro = tempfile::tempdir().expect("temporary DataRO");
    let demos = data_ro.path().join("PVsyst8.1_Data").join("Projects");
    fs::create_dir_all(&demos).expect("demo folder");
    for demo in ["_DEMO_Residential.VC5", "_DEMO_Commercial.VC3"] {
        fs::write(demos.join(demo), DEMO_CON_CARGA).expect("demo");
    }

    let mut c = Client::start_env(
        Some(ws.path().to_str().expect("utf-8 path")),
        &[(
            "PVSYST_DATARO",
            data_ro.path().to_str().expect("utf-8 path"),
        )],
    );
    c.initialize();

    let resp = c.call(
        2,
        "proyecto",
        json!({ "accion": "almacenamiento", "proyecto": "SYNTH_Plant",
                "estrategia": "autoconsumo", "bateria": "NoExiste.BTR" }),
    );
    let fallo = error_text(&resp);
    assert!(fallo.contains("NoExiste.BTR"), "error={fallo}");
    assert_eq!(
        fs::read(&vc0).unwrap(),
        VARIANTE_CON_SISTEMA.as_bytes(),
        "the variant was written despite the missing battery"
    );

    let resp = c.call(
        3,
        "proyecto",
        json!({ "accion": "almacenamiento", "proyecto": "SYNTH_Plant",
                "estrategia": "autoconsumo" }),
    );
    let out = tool_text(&resp);
    let escrito = fs::read_to_string(&vc0).unwrap();
    assert!(escrito.contains("pvGridStorage"), "out={out}");
    assert!(
        !escrito.contains("pvLoad"),
        "a load nobody asked for was inserted: {out}"
    );
}

#[test]
fn should_change_only_one_line_and_restore_exactly_when_editing_one_field() {
    let ws = synthetic_workspace();
    let vc0 = ws.path().join("Projects").join("SYNTH_Plant.VC0");
    let before = fs::read(&vc0).unwrap();
    let mut c = start(&ws);

    let edit = tool_text(&c.call(
        2,
        "editar",
        json!({
            "archivo": vc0.to_string_lossy(),
            "objeto": "pvSubArray",
            "campos": { "NStringCh": 5 }
        }),
    ));
    let historial = edit["historial_id"].as_str().expect("historial_id");

    let after = fs::read(&vc0).unwrap();
    let changed: Vec<_> = before
        .split(|b| *b == b'\n')
        .zip(after.split(|b| *b == b'\n'))
        .filter(|(a, b)| a != b)
        .collect();
    assert_eq!(changed.len(), 1, "exactly one line must change");
    assert_eq!(changed[0].1, b"      NStringCh=5\r");
    assert_eq!(before.len(), after.len(), "one digit replaced by one digit");

    let restore = c.call(
        3,
        "workspace",
        json!({ "accion": "restaurar", "id": historial }),
    );
    assert!(tool_text(&restore).is_object(), "restore must succeed");
    assert_eq!(fs::read(&vc0).unwrap(), before, "restored bytes differ");
}

#[test]
fn should_report_a_tool_error_when_a_required_argument_is_missing() {
    let ws = synthetic_workspace();
    let mut c = start(&ws);
    let resp = c.call(2, "leer", json!({}));
    assert_eq!(resp["result"]["isError"], true, "resp={resp}");
    let text = resp["result"]["content"][0]["text"].as_str().unwrap();
    assert!(text.contains("Missing: archivo"), "text={text}");
}

#[test]
fn should_reject_impossible_sizing_and_leave_the_variant_untouched_when_dimensioning() {
    let ws = synthetic_workspace();
    let vc0 = ws.path().join("Projects").join("SYNTH_Plant.VC0");
    let before = fs::read(&vc0).unwrap();
    let mut c = Client::start_env(
        Some(ws.path().to_str().unwrap()),
        &[("PVSYST_MCP_APROXIMACIONES", "1")],
    );
    for (id, args) in [
        (
            2,
            json!({ "accion": "dimensionar", "proyecto": "SYNTH_Plant", "kwp": 0.0,
                    "via": "aproximacion" }),
        ),
        (
            3,
            json!({ "accion": "dimensionar", "proyecto": "SYNTH_Plant", "kwp": 12.0,
                    "n_mod_serie": 0, "via": "aproximacion" }),
        ),
        (
            4,
            json!({ "accion": "dimensionar", "proyecto": "SYNTH_Plant", "kwp": 1e12,
                    "via": "aproximacion" }),
        ),
    ] {
        let resp = c.call(id, "proyecto", args);
        assert_eq!(resp["result"]["isError"], true, "resp={resp}");
    }
    assert_eq!(fs::read(&vc0).unwrap(), before);
}
