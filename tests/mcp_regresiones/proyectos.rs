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

/// Copies the synthetic plant to `nombre` and rewrites its variant comment.
fn plantilla_con_comentario(ws: &Path, nombre: &str, comentario: &str) {
    let proyectos = ws.join("Projects");
    fs::copy(
        proyectos.join("SYNTH_Plant.PRJ"),
        proyectos.join(format!("{nombre}.PRJ")),
    )
    .expect("prj");
    let vc0 = fs::read_to_string(proyectos.join("SYNTH_Plant.VC0"))
        .expect("vc0")
        .replace("Comment=First simulation", &format!("Comment={comentario}"));
    fs::write(proyectos.join(format!("{nombre}.VC0")), vc0).expect("variante");
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
fn should_reject_a_module_that_does_not_exist_when_dimensioning() {
    let ws = synthetic_workspace();
    let mut c = cliente(ws.path());
    let resp = c.call(
        2,
        "proyecto",
        json!({
            "accion": "dimensionar", "proyecto": "SYNTH_Plant", "variante": "VC0",
            "kwp": 6.0, "n_mod_serie": 10, "n_strings": 2, "modulo": "ZZZ_NoExiste.PAN",
        }),
    );
    let msg = error_text(&resp);
    assert!(msg.contains("ZZZ_NoExiste.PAN"), "msg={msg}");
    let vc0 = leer_proyecto(ws.path(), "SYNTH_Plant.VC0");
    assert!(
        vc0.contains("PVModule=Synth_400W.PAN"),
        "archivo alterado: {vc0}"
    );
}

#[test]
fn should_reject_module_or_inverter_on_the_gui_only_route() {
    let ws = synthetic_workspace();
    let mut c = cliente(ws.path());
    let resp = c.call(
        3,
        "proyecto",
        json!({
            "accion": "dimensionar", "proyecto": "SYNTH_Plant", "variante": "VC0",
            "kwp": 6.0, "modulo": LEGACY_PAN,
        }),
    );
    let msg = error_text(&resp);
    assert!(msg.contains("modulo"), "msg={msg}");
}

#[test]
fn should_reject_a_workspace_on_the_pvsyst_only_dimensioning_route() {
    let ws = synthetic_workspace();
    let otro = synthetic_workspace();
    let mut c = cliente(ws.path());
    let resp = c.call(
        4,
        "proyecto",
        json!({
            "accion": "dimensionar", "proyecto": "SYNTH_Plant", "variante": "VC0",
            "kwp": 6.0, "workspace": otro.path().to_string_lossy(),
        }),
    );
    let msg = error_text(&resp);
    assert!(msg.contains("no admite 'workspace'"), "msg={msg}");
}

#[test]
fn should_write_the_comment_passed_to_proyecto_crear() {
    let ws = synthetic_workspace();
    let mut c = cliente(ws.path());
    let resp = c.call(
        5,
        "proyecto",
        json!({
            "accion": "crear", "plantilla": "SYNTH_Plant", "nombre_nuevo": "E2E_Com",
            "comentario": "Mi comentario",
        }),
    );
    assert_eq!(resp["result"]["isError"], json!(false), "resp={resp}");
    assert!(leer_proyecto(ws.path(), "E2E_Com.PRJ").contains("Mi comentario"));
    assert!(leer_proyecto(ws.path(), "E2E_Com.VC0").contains("Mi comentario"));
    assert!(tool_text(&resp)["applied"].to_string().contains("Comment"));
}

#[test]
fn should_replace_a_demo_comment_with_the_project_name_when_none_is_given() {
    let ws = synthetic_workspace();
    plantilla_con_comentario(ws.path(), "DEMO_Tpl", "Residential DEMO");
    let mut c = cliente(ws.path());
    let resp = c.call(
        6,
        "proyecto",
        json!({ "accion": "crear", "plantilla": "DEMO_Tpl", "nombre_nuevo": "E2E_Dem" }),
    );
    assert_eq!(resp["result"]["isError"], json!(false), "resp={resp}");
    let vc0 = leer_proyecto(ws.path(), "E2E_Dem.VC0");
    assert!(vc0.contains("Comment=E2E_Dem"), "{vc0}");
    assert!(!vc0.contains("Residential DEMO"), "{vc0}");
}

#[test]
fn should_accept_proyecto_as_the_name_of_a_new_project() {
    let ws = synthetic_workspace();
    let mut c = cliente(ws.path());
    let resp = c.call(
        7,
        "proyecto",
        json!({ "accion": "crear", "plantilla": "SYNTH_Plant", "proyecto": "E2E_Alias" }),
    );
    assert_eq!(resp["result"]["isError"], json!(false), "resp={resp}");
    assert!(ws.path().join("Projects").join("E2E_Alias.PRJ").exists());
}

#[test]
fn should_reject_a_latitude_outside_the_valid_range() {
    let ws = synthetic_workspace();
    let mut c = cliente(ws.path());
    let resp = c.call(
        8,
        "sitio",
        json!({ "accion": "crear", "origen": "coordenadas", "latitud": 95, "longitud": 7,
                "nombre_sitio": "E2E_Lat", "workspace": ws.path().to_string_lossy() }),
    );
    assert!(error_text(&resp).contains("latitud"), "resp={resp}");
}

#[test]
fn should_reject_a_longitude_outside_the_valid_range() {
    let ws = synthetic_workspace();
    let mut c = cliente(ws.path());
    let resp = c.call(
        9,
        "sitio",
        json!({ "accion": "crear", "origen": "coordenadas", "latitud": 7, "longitud": -200,
                "nombre_sitio": "E2E_Lon", "workspace": ws.path().to_string_lossy() }),
    );
    assert!(error_text(&resp).contains("longitud"), "resp={resp}");
}

#[test]
fn should_reject_an_unknown_region_before_calling_the_cli() {
    let ws = synthetic_workspace();
    let mut c = cliente(ws.path());
    let resp = c.call(
        10,
        "sitio",
        json!({ "accion": "crear", "origen": "coordenadas", "latitud": 7, "longitud": -74,
                "nombre_sitio": "E2E_Reg", "region": "marte",
                "workspace": ws.path().to_string_lossy() }),
    );
    assert!(error_text(&resp).contains("region"), "resp={resp}");
}

#[test]
fn should_reject_a_gui_horizon_request_that_names_a_workspace() {
    let ws = synthetic_workspace();
    let otro = synthetic_workspace();
    let mut c = cliente(ws.path());
    let resp = c.call(
        11,
        "sitio",
        json!({ "accion": "horizonte_pvgis", "proyecto": "SYNTH_Plant",
                "workspace": otro.path().to_string_lossy() }),
    );
    assert!(
        error_text(&resp).contains("no admite 'workspace'"),
        "resp={resp}"
    );
}

#[test]
fn should_reject_a_grid_design_without_strings_before_any_gui_run() {
    let ws = synthetic_workspace();
    let mut c = cliente(ws.path());
    let resp = c.call(
        12,
        "disenar_sistema",
        json!({ "latitud": 4.71, "longitud": -74.07, "kwp": 5.0 }),
    );
    let msg = error_text(&resp);
    assert!(
        msg.contains("n_mod_serie") && msg.contains("n_strings"),
        "msg={msg}"
    );
}

#[test]
fn should_reject_overwriting_an_existing_component_without_sobrescribir() {
    let ws = synthetic_workspace();
    let carpeta = ws.path().join("ComposPV").join("PVModules");
    fs::write(carpeta.join("Legacy.PAN"), [0x00u8, 0x01, 0xFF]).expect("legacy");
    let mut c = cliente(ws.path());
    let mut crear = |id: u64, extra: serde_json::Value| {
        let mut args = json!({
            "accion": "crear", "plantilla": carpeta.join("Legacy.PAN").to_string_lossy(),
            "nombre_nuevo": "Copia", "tipo": "PAN", "cambios": {},
        });
        for (k, v) in extra.as_object().expect("objeto") {
            args[k.as_str()] = v.clone();
        }
        c.call(id, "componente", args)
    };
    assert_eq!(crear(13, json!({}))["result"]["isError"], json!(false));
    let msg = error_text(&crear(14, json!({})));
    assert!(msg.contains("sobrescribir"), "msg={msg}");
    assert_eq!(
        crear(15, json!({ "sobrescribir": true }))["result"]["isError"],
        json!(false)
    );
}

#[test]
fn should_reject_destino_when_importing_a_component() {
    let ws = synthetic_workspace();
    let externo = tempfile::tempdir().expect("tempdir");
    let origen = externo.path().join("Ext.PAN");
    fs::write(&origen, [0x00u8, 0x01]).expect("origen");
    let mut c = cliente(ws.path());
    let resp = c.call(
        16,
        "componente",
        json!({
            "accion": "importar", "origen": origen.to_string_lossy(),
            "destino": externo.path().join("x.PAN").to_string_lossy(), "tipo": "PAN",
        }),
    );
    let msg = error_text(&resp);
    assert!(msg.contains("destino"), "msg={msg}");
}

#[test]
fn should_filter_projects_by_texto() {
    let ws = synthetic_workspace();
    let mut c = cliente(ws.path());
    let resp = c.call(
        17,
        "listar",
        json!({ "que": "proyectos", "texto": "zzzzzz" }),
    );
    let texto = tool_text(&resp).to_string();
    assert!(!texto.contains("SYNTH_Plant"), "texto={texto}");
}

#[test]
fn should_reject_a_timeshift_outside_the_valid_range() {
    let ws = synthetic_workspace();
    let mut c = cliente(ws.path());
    let resp = c.call(
        18,
        "sitio",
        json!({ "accion": "convertir_meteo", "input_csv_file": "a.csv",
                "input_mef_file": "a.MEF", "input_sit_file": "a.SIT",
                "output_met_file": "a.MET", "input_mef_timeshift": 99, "dry_run": true }),
    );
    assert!(
        error_text(&resp).contains("input_mef_timeshift"),
        "resp={resp}"
    );
}

#[test]
fn should_name_the_missing_csv_when_converting_meteo_in_dry_run() {
    let ws = synthetic_workspace();
    let mut c = cliente(ws.path());
    let resp = c.call(
        19,
        "sitio",
        json!({ "accion": "convertir_meteo", "input_csv_file": "no_existe.csv",
                "input_mef_file": "a.MEF", "input_sit_file": "a.SIT",
                "output_met_file": "a.MET", "dry_run": true }),
    );
    assert!(error_text(&resp).contains("input_csv_file"), "resp={resp}");
}
