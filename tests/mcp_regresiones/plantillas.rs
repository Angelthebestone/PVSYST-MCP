use super::cliente_mcp::{synthetic_workspace, tool_text, Client};
use serde_json::json;
use std::fs;

#[test]
fn should_write_only_to_the_workspace_when_creating_a_project_from_a_dataro_demo_template() {
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
fn should_rewrite_the_variant_site_when_creating_a_project_from_a_site() {
    // Given: a template whose variant simulates the synthetic site, and another site
    let ws = synthetic_workspace();
    let proyectos = ws.path().join("Projects");
    let sitios = ws.path().join("Sites");
    fs::create_dir_all(&sitios).expect("Sites");
    fs::write(
        sitios.join("Lugar.SIT"),
        "PVObject_=pvSite\n  Comment=Lugar;Colombia;\n  NomF=Lugar\n  Latitude=4.7000\n  Longitude=-74.1000\nEnd of PVObject pvSite\n",
    )
    .expect("sit");

    // When
    let mut c = Client::start(Some(ws.path().to_str().expect("ws")));
    c.initialize();
    let resp = c.call(
        1,
        "proyecto",
        json!({ "accion": "crear", "plantilla": "SYNTH_Plant", "nombre_nuevo": "Con_Sitio", "sitio": "Lugar" }),
    );
    let out = tool_text(&resp);
    assert_eq!(resp["result"]["isError"], false, "resp={resp}");

    // Then: project and variant both carry the requested site, not the template's
    for archivo in ["Con_Sitio.PRJ", "Con_Sitio.VC0"] {
        let texto = fs::read_to_string(proyectos.join(archivo)).expect(archivo);
        assert!(
            texto.contains("Comment=Lugar;Colombia;"),
            "{archivo}={texto}"
        );
        assert!(
            !texto.contains("Comment=Synthetic;Nowhere;"),
            "{archivo} keeps the template site: {texto}"
        );
    }
    assert_eq!(out["variant_site_updated"], json!(true), "out={out}");
}
