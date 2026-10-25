use crate::server::tools::recorte::{decorar, siguiente, Respuesta};
use serde_json::json;

fn respuesta(
    herramienta: &'static str,
    accion: &'static str,
    ignorados: Vec<String>,
) -> Respuesta<'static> {
    Respuesta {
        herramienta,
        accion,
        ms: 1,
        ignorados,
    }
}

#[test]
fn should_mark_a_trimmed_answer_with_truncado_and_hint() {
    let mut v = json!({ "items": (0..20).collect::<Vec<u32>>() });
    decorar(
        &mut v,
        &json!({}),
        respuesta("listar", "proyectos", Vec::new()),
    );
    assert_eq!(v["items"].as_array().map(Vec::len), Some(13));
    assert_eq!(v["items_total"], 20);
    assert_eq!(v["truncado"], true);
    assert_eq!(v["hint"], "detalle:completo");
}

#[test]
fn should_report_the_original_length_as_top_level_total_when_trimmed() {
    let mut v = json!({ "items": (0..20).collect::<Vec<u32>>() });
    decorar(
        &mut v,
        &json!({}),
        respuesta("listar", "proyectos", Vec::new()),
    );
    assert_eq!(v["total"], 20);
    assert_eq!(v["items_total"], 20);
}

#[test]
fn should_omit_top_level_total_when_nothing_was_trimmed() {
    let mut v = json!({ "items": (0..3).collect::<Vec<u32>>() });
    decorar(
        &mut v,
        &json!({}),
        respuesta("listar", "proyectos", Vec::new()),
    );
    assert!(v.get("total").is_none());
}

#[test]
fn should_keep_every_item_and_no_marker_with_detalle_completo() {
    let mut v = json!({ "items": (0..20).collect::<Vec<u32>>() });
    decorar(
        &mut v,
        &json!({ "detalle": "completo" }),
        respuesta("listar", "proyectos", Vec::new()),
    );
    assert_eq!(v["items"].as_array().map(Vec::len), Some(20));
    assert!(v.get("truncado").is_none());
    assert!(v.get("hint").is_none());
}

#[test]
fn should_list_ignored_arguments_only_when_there_are_some() {
    let mut con = json!({ "ok": true });
    decorar(
        &mut con,
        &json!({}),
        respuesta("listar", "proyectos", vec!["texto".to_string()]),
    );
    assert_eq!(con["ignorados"], json!(["texto"]));

    let mut sin = json!({ "ok": true });
    decorar(
        &mut sin,
        &json!({}),
        respuesta("listar", "proyectos", Vec::new()),
    );
    assert!(sin.get("ignorados").is_none());
}

#[test]
fn should_suggest_the_next_call_after_creating_a_project() {
    let mut v = json!({ "ok": true });
    decorar(
        &mut v,
        &json!({}),
        respuesta("proyecto", "crear", Vec::new()),
    );
    let siguiente = v["siguiente"].as_str().expect("siguiente");
    assert!(siguiente.starts_with("proyecto dimensionar"), "{siguiente}");
}

#[test]
fn should_point_kpis_to_the_losses_query_instead_of_itself() {
    assert_eq!(
        siguiente("resultados", "kpis"),
        Some("resultados que:perdidas")
    );
}
