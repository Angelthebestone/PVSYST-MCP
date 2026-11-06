use super::cliente_mcp::{error_text, synthetic_workspace, tool_text, Client};
use serde_json::json;
use std::fs;
use std::path::Path;

const LEGACY_PAN: &str = "Synth_Legacy_300W.PAN";
const SINTETICO_OND: &str = "Synth_10kW.OND";

fn cliente(ws: &Path) -> Client {
    let ws_str = ws.to_string_lossy().to_string();
    let mut c = Client::start(Some(&ws_str));
    c.initialize();
    c
}

fn leer_proyecto(ws: &Path, archivo: &str) -> String {
    fs::read_to_string(ws.join("Projects").join(archivo)).expect(archivo)
}

#[test]
fn should_apply_module_and_inverter_when_dimensioning_a_variant() {
    let ws = synthetic_workspace();
    let mut c = cliente(ws.path());
    let resp = c.call(
        1,
        "proyecto",
        json!({
            "accion": "dimensionar", "proyecto": "SYNTH_Plant", "variante": "VC0",
            "kwp": 6.0, "n_mod_serie": 10, "n_strings": 2,
            "modulo": LEGACY_PAN, "inversor": SINTETICO_OND,
        }),
    );
    assert_eq!(resp["result"]["isError"], json!(false), "resp={resp}");
    let vc0 = leer_proyecto(ws.path(), "SYNTH_Plant.VC0");
    assert!(vc0.contains(&format!("PVModule={LEGACY_PAN}")), "{vc0}");
    assert!(vc0.contains(&format!("GInverter={SINTETICO_OND}")), "{vc0}");
    let applied = tool_text(&resp)["applied"].to_string();
    assert!(applied.contains("PVModule"), "applied={applied}");
    assert!(applied.contains("GInverter"), "applied={applied}");
}

#[test]
fn should_apply_every_step_when_configurar_chains_variant_changes() {
    let ws = synthetic_workspace();
    let mut c = cliente(ws.path());
    let resp = c.call(
        1,
        "proyecto",
        json!({
            "accion": "configurar", "proyecto": "SYNTH_Plant", "variante": "VC0",
            "pasos": [
                { "accion": "dimensionar", "kwp": 6.0, "n_mod_serie": 10, "n_strings": 2 },
                { "accion": "dimensionar", "kwp": 6.0, "n_mod_serie": 8, "n_strings": 3 },
            ],
        }),
    );
    assert_eq!(resp["result"]["isError"], json!(false), "resp={resp}");
    assert_eq!(tool_text(&resp)["pasos_aplicados"], json!(2), "resp={resp}");
    let vc0 = leer_proyecto(ws.path(), "SYNTH_Plant.VC0");
    assert!(vc0.contains("NModSerie=8"), "{vc0}");
    assert!(vc0.contains("NStringCh=3"), "{vc0}");
}

#[test]
fn should_reject_a_configurar_step_that_is_not_a_variant_change_before_running_any() {
    let ws = synthetic_workspace();
    let antes = leer_proyecto(ws.path(), "SYNTH_Plant.VC0");
    let mut c = cliente(ws.path());
    let resp = c.call(
        1,
        "proyecto",
        json!({
            "accion": "configurar", "proyecto": "SYNTH_Plant", "variante": "VC0",
            "pasos": [
                { "accion": "dimensionar", "kwp": 6.0, "n_mod_serie": 8, "n_strings": 3 },
                { "accion": "eliminar" },
            ],
        }),
    );
    assert_eq!(resp["result"]["isError"], json!(true), "resp={resp}");
    assert!(error_text(&resp).contains("eliminar"), "resp={resp}");
    assert_eq!(leer_proyecto(ws.path(), "SYNTH_Plant.VC0"), antes);
}
