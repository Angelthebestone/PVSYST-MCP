use super::cliente_mcp::{error_text, synthetic_workspace, tool_text, Client};
use serde_json::json;
use std::fs;

fn servidor(ws: &tempfile::TempDir) -> Client {
    let mut c = Client::start(Some(&ws.path().to_string_lossy()));
    c.initialize();
    c
}

#[test]
fn should_refuse_to_read_empty_or_truncated_projects_of_the_requested_workspace() {
    let ws = synthetic_workspace();
    let proyectos = ws.path().join("Projects");
    fs::write(proyectos.join("Vacio.PRJ"), "").unwrap();
    fs::write(
        proyectos.join("Cortado.PRJ"),
        "PVObject_=pvProject\n  Comment=cortado\n",
    )
    .unwrap();
    // The server's own workspace is another one: names must resolve in `workspace`.
    let propio = synthetic_workspace();
    let mut c = servidor(&propio);
    let pedido = ws.path().to_string_lossy().to_string();

    let msg = error_text(&c.call(
        2,
        "leer",
        json!({ "archivo": "Vacio", "workspace": &pedido }),
    ));
    assert!(msg.contains("Archivo vacío"), "{msg}");
    let msg = error_text(&c.call(
        3,
        "leer",
        json!({ "archivo": "Cortado", "workspace": &pedido }),
    ));
    assert!(msg.contains("Archivo truncado"), "{msg}");
}

#[test]
fn should_anchor_relative_output_paths_to_the_workspace() {
    let ws = synthetic_workspace();
    let mut c = servidor(&ws);
    let dado = tool_text(&c.call(
        2,
        "simular",
        json!({ "proyecto": "SYNTH_Plant", "output_csv": "salidas/a.csv",
                "report_pdf": "b.pdf", "dry_run": true }),
    ));
    let csv = dado["output_csv"].as_str().unwrap();
    let pdf = dado["report_pdf"].as_str().unwrap();
    assert!(
        std::path::Path::new(csv).starts_with(ws.path()),
        "data={dado}"
    );
    assert!(csv.ends_with("a.csv"), "data={dado}");
    assert!(
        std::path::Path::new(pdf).starts_with(ws.path()),
        "data={dado}"
    );
}
#[test]
fn should_report_the_trial_cost_of_a_p50_p90_dry_run_without_opening_pvsyst() {
    // Given: the synthetic project, whose variant file exists
    let ws = synthetic_workspace();
    let mut c = servidor(&ws);

    // When
    let dado = tool_text(&c.call(
        1,
        "proyecto",
        json!({ "accion": "p50_p90", "proyecto": "SYNTH_Plant", "dry_run": true }),
    ));

    // Then: the plan and its cost, and PVsyst was not opened
    assert_eq!(dado["dry_run"], true, "data={dado}");
    assert_eq!(dado["variant_file_exists"], true, "data={dado}");
    assert_eq!(dado["costo_estimado"]["ejecuciones"], 0, "data={dado}");
    assert_eq!(dado["requiere_gui"], true, "data={dado}");
}

#[test]
fn should_refuse_a_dry_run_on_a_project_action_that_would_run_for_real() {
    let ws = synthetic_workspace();
    let mut c = servidor(&ws);
    let msg = error_text(&c.call(
        1,
        "proyecto",
        json!({ "accion": "dimensionar", "proyecto": "SYNTH_Plant", "kwp": 8.0,
                "dry_run": true }),
    ));
    assert!(msg.contains("dry_run"), "msg={msg}");
    assert!(msg.contains("dimensionar"), "msg={msg}");
    assert!(
        msg.contains("no tiene ensayo") || msg.contains("would run for real"),
        "{msg}"
    );
}

#[test]
fn should_state_the_real_cost_when_horizonte_pvgis_refuses_a_dry_run() {
    let ws = synthetic_workspace();
    let mut c = servidor(&ws);
    let msg = error_text(&c.call(
        1,
        "sitio",
        json!({ "accion": "horizonte_pvgis", "proyecto": "SYNTH_Plant", "dry_run": true }),
    ));
    assert!(msg.contains("no dry run"), "msg={msg}");
    assert!(msg.contains("trial executions"), "msg={msg}");
}

#[test]
fn should_report_the_trial_cost_of_a_design_dry_run_without_doing_any_step() {
    let ws = synthetic_workspace();
    let mut c = servidor(&ws);
    let dado = tool_text(&c.call(
        1,
        "disenar_sistema",
        json!({ "latitud": 10.0, "longitud": -70.0, "kwp": 8.0,
                "n_mod_serie": 10, "n_strings": 2, "dry_run": true }),
    ));
    assert_eq!(dado["dry_run"], true, "data={dado}");
    assert_eq!(dado["costo_estimado"]["ejecuciones"], 2, "data={dado}");
    assert!(
        dado["plan"].as_array().is_some_and(|p| !p.is_empty()),
        "data={dado}"
    );
    assert!(
        !ws.path().join("Projects").join("Sistema_8kWp.PRJ").exists(),
        "a dry run must not create the project"
    );
}
