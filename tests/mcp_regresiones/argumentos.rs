use super::cliente_mcp::{error_text, synthetic_workspace, tool_text, Client};
use serde_json::json;
use std::fs;

fn servidor(ws: &tempfile::TempDir) -> Client {
    let mut c = Client::start(Some(&ws.path().to_string_lossy()));
    c.initialize();
    c
}

#[test]
fn should_refuse_an_empty_contenido() {
    let ws = synthetic_workspace();
    let mut c = servidor(&ws);
    let resp = c.call(
        2,
        "escribir_archivo",
        json!({ "archivo": "UserData/vacio.txt", "contenido": "" }),
    );
    let msg = error_text(&resp);
    assert!(msg.contains("must not be empty"), "{msg}");
    assert!(!ws.path().join("UserData/vacio.txt").exists());
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
fn should_reject_a_log_level_outside_range_on_convert_meteo() {
    let ws = synthetic_workspace();
    for f in ["in.csv", "in.mef", "in.sit"] {
        std::fs::write(ws.path().join(f), "x").expect("escribir entrada");
    }
    let mut c = servidor(&ws);
    let msg = error_text(&c.call(
        2,
        "sitio",
        json!({ "accion": "convertir_meteo", "input_csv_file": "in.csv",
                "input_mef_file": "in.mef", "input_sit_file": "in.sit",
                "output_met_file": "out.met", "log_level": 9, "dry_run": true }),
    ));
    assert!(msg.contains("log_level"), "{msg}");
}
