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
