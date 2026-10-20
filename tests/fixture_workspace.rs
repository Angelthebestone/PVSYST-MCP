//! Hermetic end-to-end tests: the server runs against the synthetic workspace in
//! `tests/fixtures/workspace`, so they need no PVsyst installation, network or clock.
// Test code: failing loudly on unexpected state is the point.
#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::too_many_lines,
    clippy::needless_pass_by_value,
    unreachable_pub
)]

mod common;

use common::{synthetic_workspace, tool_text, Client};
use serde_json::json;
use std::fs;

fn start(ws: &tempfile::TempDir) -> Client {
    let mut c = Client::start(Some(ws.path().to_str().expect("utf-8 path")));
    c.initialize();
    c
}

#[test]
fn listar_proyectos_finds_the_synthetic_project_and_its_variant() {
    let ws = synthetic_workspace();
    let mut c = start(&ws);
    let data = tool_text(&c.call(2, "listar", json!({ "que": "proyectos" })));
    assert_eq!(data["count"], 1, "data={data}");
    assert_eq!(data["projects"][0]["name"], "SYNTH_Plant");
    assert_eq!(data["projects"][0]["variants"], json!(["VC0"]));
}

#[test]
fn simular_dry_run_builds_unquoted_arguments() {
    let ws = synthetic_workspace();
    let mut c = start(&ws);
    let resp = c.call(
        2,
        "simular",
        json!({ "proyecto": "SYNTH_Plant", "variante": "VC0", "dry_run": true }),
    );
    let data = tool_text(&resp);
    let cmd = data["command_line"].as_str().unwrap();
    assert_eq!(data["variant_file_exists"], true, "data={data}");
    assert!(cmd.contains("-w:"), "cmd was {cmd}");
    assert!(
        !cmd.contains("-w:\""),
        "workspace must not be quoted: {cmd}"
    );
    assert!(cmd.contains("-p:SYNTH_Plant.PRJ"), "cmd was {cmd}");
}

#[test]
fn editing_one_field_changes_only_that_line_and_restores_exactly() {
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
fn proyecto_crear_sets_the_module_reference() {
    let ws = synthetic_workspace();
    let mut c = start(&ws);
    let resp = c.call(
        2,
        "proyecto",
        json!({
            "accion": "crear",
            "plantilla": "SYNTH_Plant",
            "nombre_nuevo": "Copia",
            "modulo": "Otro_450W.PAN"
        }),
    );
    assert_eq!(resp["result"]["isError"], false, "resp={resp}");
    let vc = fs::read_to_string(ws.path().join("Projects").join("Copia.VC0")).unwrap();
    assert!(vc.contains("PVModule=Otro_450W.PAN"));
    assert!(!vc.contains("PVModule=Synth_400W.PAN"));
    assert!(ws.path().join("Projects").join("Copia.PRJ").exists());
}

#[test]
fn proyecto_dimensionar_sizes_the_array_from_the_module_power() {
    let ws = synthetic_workspace();
    let mut c = start(&ws);
    let resp = c.call(
        2,
        "proyecto",
        json!({ "accion": "dimensionar", "proyecto": "SYNTH_Plant", "kwp": 12.0 }),
    );
    let data = tool_text(&resp);
    // 12 kWp / 400 W = 30 modules; 10 in series (from the variant) -> 3 strings.
    assert_eq!(data["n_strings"], 3, "data={data}");
    assert_eq!(data["realised_kwp"], 12.0);
    let vc = fs::read_to_string(ws.path().join("Projects").join("SYNTH_Plant.VC0")).unwrap();
    assert!(vc.contains("NStringCh=3"));
}

#[test]
fn proyecto_dimensionar_rejects_impossible_sizing_and_leaves_the_variant_untouched() {
    let ws = synthetic_workspace();
    let vc0 = ws.path().join("Projects").join("SYNTH_Plant.VC0");
    let before = fs::read(&vc0).unwrap();
    let mut c = start(&ws);
    for (id, args) in [
        (
            2,
            json!({ "accion": "dimensionar", "proyecto": "SYNTH_Plant", "kwp": 0.0 }),
        ),
        (
            3,
            json!({ "accion": "dimensionar", "proyecto": "SYNTH_Plant", "kwp": 12.0, "n_mod_serie": 0 }),
        ),
        (
            4,
            json!({ "accion": "dimensionar", "proyecto": "SYNTH_Plant", "kwp": 1e12 }),
        ),
    ] {
        let resp = c.call(id, "proyecto", args);
        assert_eq!(resp["result"]["isError"], true, "resp={resp}");
    }
    assert_eq!(fs::read(&vc0).unwrap(), before);
}

#[test]
fn leer_decodes_the_text_module() {
    let ws = synthetic_workspace();
    let pan = ws.path().join("ComposPV/PVModules/Synth_400W.PAN");
    let mut c = start(&ws);
    let data = tool_text(&c.call(2, "leer", json!({ "archivo": pan.to_string_lossy() })));
    assert_eq!(data["vendor"], "Synthetic", "data={data}");
    assert_eq!(data["pnom"].as_f64(), Some(400.0));
    assert_eq!(data["voc"].as_f64(), Some(49.5));
}

#[test]
fn a_missing_required_argument_is_reported_as_a_tool_error() {
    let ws = synthetic_workspace();
    let mut c = start(&ws);
    let resp = c.call(2, "leer", json!({}));
    assert_eq!(resp["result"]["isError"], true, "resp={resp}");
    let text = resp["result"]["content"][0]["text"].as_str().unwrap();
    assert!(
        text.contains("Missing required argument 'archivo'"),
        "text={text}"
    );
}

#[test]
fn doctor_reports_cold_voc_above_inverter_limit_for_long_strings() {
    let ws = synthetic_workspace();
    let prj = ws.path().join("Projects").join("SYNTH_Plant.PRJ");
    let texto = fs::read_to_string(&prj).unwrap().replacen(
        "MeteoFileName=",
        "DesignTempMin=-10\r\nTempOperMax=70\r\nMeteoFileName=",
        1,
    );
    fs::write(&prj, texto).unwrap();
    let vc0 = ws.path().join("Projects").join("SYNTH_Plant.VC0");
    let variante = fs::read_to_string(&vc0)
        .unwrap()
        .replace("NModSerie=10", "NModSerie=25");
    fs::write(&vc0, variante).unwrap();
    let mut c = start(&ws);

    let informe = tool_text(&c.call(
        2,
        "proyecto",
        json!({ "accion": "doctor", "proyecto": "SYNTH_Plant" }),
    ));

    let errores = informe["errores"].to_string();
    assert!(errores.contains("Voc en frío"), "informe={informe}");
}
