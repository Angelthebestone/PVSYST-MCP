use super::cliente_mcp::{error_text, synthetic_workspace, tool_text, Client};
use serde_json::{json, Value};
use std::fs;

fn servidor(ws: &tempfile::TempDir) -> Client {
    let mut c = Client::start(Some(&ws.path().to_string_lossy()));
    c.initialize();
    c
}

fn herramienta(tools: &Value, nombre: &str) -> Value {
    tools["result"]["tools"]
        .as_array()
        .and_then(|todas| todas.iter().find(|t| t["name"] == nombre))
        .cloned()
        .unwrap_or_else(|| panic!("sin la herramienta {nombre}"))
}

#[test]
fn should_publish_aliases_a_validating_client_accepts() {
    let ws = synthetic_workspace();
    let mut c = servidor(&ws);
    let tools = c.request(2, "tools/list", json!({}));

    let que = &herramienta(&tools, "listar")["inputSchema"]["properties"]["que"]["enum"];
    assert!(
        que.as_array().unwrap().contains(&json!("modules")),
        "enum={que}"
    );
    assert!(
        que.as_array().unwrap().contains(&json!("modulos")),
        "enum={que}"
    );

    let leer = herramienta(&tools, "leer");
    let requeridos = leer["inputSchema"]["required"].as_array().unwrap();
    assert!(
        !requeridos.contains(&json!("archivo")),
        "required={requeridos:?}"
    );
    let descripcion = leer["inputSchema"]["properties"]["archivo"]["description"].as_str();
    assert!(
        descripcion.unwrap().contains("Requerido"),
        "{descripcion:?}"
    );
}

#[test]
fn should_describe_listar_texto_as_a_search_filter_not_gui_input() {
    let ws = synthetic_workspace();
    let mut c = servidor(&ws);
    let tools = c.request(2, "tools/list", json!({}));
    let descripcion = herramienta(&tools, "listar")["inputSchema"]["properties"]["texto"]
        ["description"]
        .to_string();
    assert!(!descripcion.contains("combo"), "descripcion={descripcion}");
}

#[test]
fn should_write_an_empty_file() {
    let ws = synthetic_workspace();
    let mut c = servidor(&ws);
    let resp = c.call(
        2,
        "escribir_archivo",
        json!({ "archivo": "UserData/vacio.txt", "contenido": "" }),
    );
    assert_eq!(tool_text(&resp)["bytes_written"], 0, "resp={resp}");
    assert_eq!(
        fs::metadata(ws.path().join("UserData/vacio.txt"))
            .unwrap()
            .len(),
        0
    );
}

#[test]
fn should_name_a_non_numeric_latitude_instead_of_calling_it_missing() {
    let ws = synthetic_workspace();
    let mut c = servidor(&ws);
    let resp = c.call(
        2,
        "sitio",
        json!({ "accion": "crear", "nombre_sitio": "Z", "latitud": "abc", "longitud": -74, "dry_run": true }),
    );
    let msg = error_text(&resp);
    assert!(
        msg.contains("'latitud' debe ser un número finito") && msg.contains("abc"),
        "{msg}"
    );
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
fn should_report_the_resolved_csv_once_in_a_dry_run() {
    let ws = synthetic_workspace();
    let mut c = servidor(&ws);
    let dado = tool_text(&c.call(
        2,
        "simular",
        json!({ "proyecto": "SYNTH_Plant", "csv": "a.csv", "dry_run": true }),
    ));
    let csv = dado["output_csv"].as_str().unwrap();
    assert!(csv.ends_with("a.csv"), "data={dado}");
    assert!(
        std::path::Path::new(csv).starts_with(ws.path()),
        "data={dado}"
    );
    let ocf = dado["cli_args"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|a| a.as_str().unwrap().starts_with("-ocf:"));
    assert_eq!(ocf.count(), 1, "data={dado}");

    let por_defecto = tool_text(&c.call(
        3,
        "simular",
        json!({ "proyecto": "SYNTH_Plant", "dry_run": true }),
    ));
    let csv = por_defecto["output_csv"].as_str().unwrap();
    assert!(csv.ends_with("SYNTH_Plant_VC0.csv"), "data={por_defecto}");
}

#[test]
fn should_reject_a_count_below_its_declared_minimum() {
    let ws = synthetic_workspace();
    let mut c = servidor(&ws);
    let msg = error_text(&c.call(2, "workspace", json!({ "accion": "historial", "n": -1 })));
    assert!(msg.contains("fuera de rango para 'n'"), "{msg}");
}

#[test]
fn should_reject_a_tool_call_without_a_string_name() {
    let ws = synthetic_workspace();
    let mut c = servidor(&ws);
    let resp = c.request(2, "tools/call", json!({ "name": 5, "arguments": {} }));
    assert!(error_text(&resp).contains("'name'"), "resp={resp}");
}

fn error_de_simular(ws: &tempfile::TempDir, args: Value) -> String {
    let mut c = servidor(ws);
    error_text(&c.call(2, "simular", args))
}

#[test]
fn should_reject_a_time_step_outside_the_documented_values() {
    let ws = synthetic_workspace();
    let msg = error_de_simular(
        &ws,
        json!({ "proyecto": "SYNTH_Plant", "time_step": "minute", "dry_run": true }),
    );
    assert!(msg.contains("time_step"), "{msg}");
}

#[test]
fn should_reject_a_calendar_impossible_start_date() {
    let ws = synthetic_workspace();
    let msg = error_de_simular(
        &ws,
        json!({ "proyecto": "SYNTH_Plant", "start_date": "2024-02-31", "dry_run": true }),
    );
    assert!(msg.contains("start_date"), "{msg}");
}

#[test]
fn should_reject_an_end_date_before_the_start_date() {
    let ws = synthetic_workspace();
    let msg = error_de_simular(
        &ws,
        json!({ "proyecto": "SYNTH_Plant", "start_date": "2024-06-01",
                "end_date": "2024-05-01", "dry_run": true }),
    );
    assert!(msg.contains("end_date"), "{msg}");
}

#[test]
fn should_reject_an_unknown_report_page_naming_the_parameter() {
    let ws = synthetic_workspace();
    let msg = error_de_simular(
        &ws,
        json!({ "proyecto": "SYNTH_Plant", "paginas_reporte": ["zzz"], "dry_run": true }),
    );
    assert!(
        msg.contains("paginas_reporte") && msg.contains("zzz"),
        "{msg}"
    );
}

#[test]
fn should_reject_a_log_level_outside_zero_to_three() {
    let ws = synthetic_workspace();
    let msg = error_de_simular(
        &ws,
        json!({ "proyecto": "SYNTH_Plant", "log_level": 9, "dry_run": true }),
    );
    assert!(msg.contains("log_level"), "{msg}");
}

#[test]
fn should_reject_a_variant_that_is_not_on_disk_and_list_the_existing_ones() {
    let ws = synthetic_workspace();
    let msg = error_de_simular(
        &ws,
        json!({ "proyecto": "SYNTH_Plant", "variante": "CM1", "dry_run": true }),
    );
    assert!(msg.contains("CM1") && msg.contains("VC0"), "{msg}");
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

#[test]
fn should_not_list_approximated_results_actions_when_approximations_are_off() {
    let ws = synthetic_workspace();
    let mut c = servidor(&ws);
    let msg = error_text(&c.call(2, "resultados", json!({ "que": "zzz" })));
    assert!(msg.contains("kpis"), "{msg}");
    assert!(!msg.contains("medido_vs_simulado"), "{msg}");
}

#[test]
fn should_require_a_project_or_pdf_for_resultados_kpis() {
    let ws = synthetic_workspace();
    let mut c = servidor(&ws);
    let msg = error_text(&c.call(2, "resultados", json!({ "que": "kpis" })));
    assert!(msg.contains("'proyecto'"), "{msg}");
}

#[test]
fn should_report_unknown_project_for_resultados_kpis_as_project_not_found() {
    let ws = synthetic_workspace();
    let mut c = servidor(&ws);
    let msg = error_text(&c.call(
        2,
        "resultados",
        json!({ "que": "kpis", "proyecto": "NoExiste" }),
    ));
    assert!(
        msg.contains("Proyecto") && msg.contains("no encontrado"),
        "{msg}"
    );
    assert!(!msg.contains("Informe PDF"), "{msg}");
}

#[test]
fn should_reject_include_text_outside_resultados_csv() {
    let ws = synthetic_workspace();
    let mut c = servidor(&ws);
    let msg = error_text(&c.call(
        2,
        "resultados",
        json!({ "que": "kpis", "proyecto": "X", "include_text": true }),
    ));
    assert!(msg.contains("include_text"), "{msg}");
}
