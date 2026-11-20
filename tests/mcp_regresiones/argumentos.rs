use super::cliente_mcp::{error_text, synthetic_workspace, tool_text, Client};
use serde_json::{json, Value};
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

/// The `vista` block of a failed call: the client negotiates `2024-11-05`, so the
/// server writes it as a second text item instead of `structuredContent`.
fn vista_del_fallo(resp: &Value) -> Option<Value> {
    resp.get("result")?
        .get("content")?
        .as_array()?
        .iter()
        .filter_map(|item| item.get("text").and_then(Value::as_str))
        .filter_map(|texto| serde_json::from_str::<Value>(texto).ok())
        .find_map(|bloque| bloque.get("vista").cloned())
}

/// B.5.1 of the visual GUI test report: `vista:false` travels the whole wire —
/// `preparar_args` keeps it, `validar` type-checks it and `declara` does not call
/// it ignored — and drops the block only from a call that finished.
#[test]
fn should_drop_the_view_only_from_a_finished_call() {
    let ws = synthetic_workspace();
    let mut c = servidor(&ws);

    let datos = tool_text(&c.call(40, "listar", json!({ "que": "proyectos", "vista": false })));
    assert!(
        datos.get("vista").is_none(),
        "una llamada terminada lo omite: {datos}"
    );
    assert!(
        datos.get("ignorados").is_none(),
        "'vista' no es un argumento ignorado: {datos}"
    );

    let negada = error_text(&c.call(41, "listar", json!({ "que": "proyectos", "vista": "x" })));
    assert!(negada.contains("vista"), "{negada}");
    assert!(negada.contains("boolean"), "{negada}");

    let fallo = c.call(
        42,
        "leer",
        json!({ "archivo": "Projects/NO.PRJ", "vista": false }),
    );
    let vista = vista_del_fallo(&fallo).unwrap_or_else(|| panic!("sin vista: {fallo}"));
    assert_eq!(vista["estado"], json!("error"), "{fallo}");
}
