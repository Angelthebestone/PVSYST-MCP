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

/// An inventory asks for every control: the summary level that keeps 13 items
/// would hide most of a form, so it keeps up to 500 and says how many exist.
#[test]
fn should_keep_up_to_500_controls_when_a_gui_inventory_is_asked_without_detail() {
    let controles = |n: u32| json!({ "count": n, "controls": (0..n).collect::<Vec<u32>>() });
    let pedido = json!({ "que": "controles", "todos": true });

    let mut cabe = controles(300);
    decorar(
        &mut cabe,
        &pedido,
        respuesta("gui_observar", "controles", Vec::new()),
    );
    assert_eq!(cabe["controls"].as_array().map(Vec::len), Some(300));
    assert!(cabe.get("truncado").is_none());

    let mut excede = controles(520);
    decorar(
        &mut excede,
        &pedido,
        respuesta("gui_observar", "controles", Vec::new()),
    );
    assert_eq!(excede["controls"].as_array().map(Vec::len), Some(500));
    assert_eq!(
        (excede["truncado"].as_bool(), excede["total"].as_u64()),
        (Some(true), Some(520))
    );

    let mut con_detalle = controles(300);
    let resumen = json!({ "que": "controles", "todos": true, "detalle": "resumen" });
    decorar(
        &mut con_detalle,
        &resumen,
        respuesta("gui_observar", "controles", Vec::new()),
    );
    assert_eq!(con_detalle["controls"].as_array().map(Vec::len), Some(13));
}

/// The inventory and the form view carry their own cap and counts: the generic
/// summary must not cut their lists.
#[test]
fn should_not_summarise_the_lists_of_the_inventory_and_the_form_view() {
    let mut v = json!({ "controles": (0..40).collect::<Vec<u32>>(), "otros": (0..40).collect::<Vec<u32>>() });
    decorar(
        &mut v,
        &json!({}),
        respuesta("gui_observar", "inventario", Vec::new()),
    );
    assert_eq!(v["controles"].as_array().map(Vec::len), Some(40));
    assert_eq!(v["otros"].as_array().map(Vec::len), Some(40));
    assert!(v.get("truncado").is_none());
}
