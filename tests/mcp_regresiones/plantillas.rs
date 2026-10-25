use super::cliente_mcp::{synthetic_workspace, tool_text, Client};
use serde_json::json;
use std::fs;

#[test]
fn should_create_project_from_a_datar_demo_template_writing_only_to_the_workspace() {
    let ws = synthetic_workspace();
    let data_ro = tempfile::tempdir().expect("DataRO temporal");
    let demos = data_ro.path().join("PVsyst8.1_Data").join("Projects");
    fs::create_dir_all(&demos).expect("carpeta de demos");
    let proyectos = ws.path().join("Projects");
    fs::copy(
        proyectos.join("SYNTH_Plant.PRJ"),
        demos.join("DEMO_Ref.PRJ"),
    )
    .expect("prj demo");
    fs::copy(
        proyectos.join("SYNTH_Plant.VC0"),
        demos.join("DEMO_Ref.VC0"),
    )
    .expect("vc0 demo");

    let ws_str = ws.path().to_string_lossy().to_string();
    let data_str = data_ro.path().to_string_lossy().to_string();
    let mut c = Client::start_env(Some(&ws_str), &[("PVSYST_DATARO", data_str.as_str())]);
    c.initialize();
    let resp = c.call(
        1,
        "proyecto",
        json!({ "accion": "crear", "plantilla": "DEMO_Ref", "nombre_nuevo": "Copia_Demo" }),
    );
    let out = tool_text(&resp);
    assert!(proyectos.join("Copia_Demo.PRJ").exists(), "resp={out}");
    assert!(
        !demos.join("Copia_Demo.PRJ").exists(),
        "DataRO escrito: resp={out}"
    );
}

#[test]
fn should_link_the_requested_workspace_site_when_creating_a_project_with_sitio() {
    let ws = synthetic_workspace();
    let sites = ws.path().join("Sites");
    fs::create_dir_all(&sites).expect("carpeta de sitios");
    fs::write(
        sites.join("SITIO_Test.SIT"),
        "PVObject_=pvSite\r\n  Name=SITIO_Test\r\nEnd of PVObject pvSite\r\n",
    )
    .expect("sitio");

    let ws_str = ws.path().to_string_lossy().to_string();
    let mut c = Client::start(Some(&ws_str));
    c.initialize();
    let resp = c.call(
        1,
        "proyecto",
        json!({ "accion": "crear", "plantilla": "SYNTH_Plant", "nombre_nuevo": "Copia_Sitio", "sitio": "SITIO_Test" }),
    );
    let out = tool_text(&resp);
    assert!(out["sitio"].is_string(), "resp={out}");
    let prj = fs::read_to_string(ws.path().join("Projects").join("Copia_Sitio.PRJ")).expect("PRJ");
    assert!(prj.contains("SITIO_Test"), "PRJ sin el sitio pedido");
}

#[test]
fn should_reject_a_sitio_missing_from_the_workspace_before_writing_the_project() {
    let ws = synthetic_workspace();
    let ws_str = ws.path().to_string_lossy().to_string();
    let mut c = Client::start(Some(&ws_str));
    c.initialize();
    let resp = c.call(
        1,
        "proyecto",
        json!({ "accion": "crear", "plantilla": "SYNTH_Plant", "nombre_nuevo": "Sin_Sitio", "sitio": "NO_EXISTE" }),
    );
    assert_eq!(resp["result"]["isError"], json!(true), "resp={resp}");
    assert!(!ws.path().join("Projects").join("Sin_Sitio.PRJ").exists());
}
