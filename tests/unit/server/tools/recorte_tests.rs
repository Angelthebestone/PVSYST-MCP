use crate::server::tools::recorte::{decorar, Respuesta};
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
fn should_mark_the_answer_as_trimmed_when_the_detail_level_cuts_items() {
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
fn should_keep_every_item_when_detalle_is_completo() {
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
