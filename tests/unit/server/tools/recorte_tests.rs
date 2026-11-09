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

/// BUG-16: `count` kept the length before the cut while the list had 13 items.
#[test]
fn should_make_count_say_how_many_items_are_left_when_the_list_it_counts_is_cut() {
    let mut v = json!({ "count": 24, "controls": (0..24).collect::<Vec<u32>>() });
    decorar(
        &mut v,
        &json!({}),
        respuesta("gui_observar", "controles", Vec::new()),
    );
    assert_eq!(v["controls"].as_array().map(Vec::len), Some(13));
    assert_eq!(v["count"], 13);
    assert_eq!(v["controls_total"], 24);
}
