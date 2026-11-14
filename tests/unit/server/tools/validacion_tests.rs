use super::*;
use crate::config::Paths;

#[test]
fn should_require_operacion_when_editing_rvt() {
    let paths = Paths::para_test(std::env::temp_dir());
    let err = call(&paths, "editar", &json!({ "accion": "rvt" }))
        .expect_err("rvt without operacion must not default to listar");
    assert!(err.to_string().contains("operacion"), "{err}");
}

#[test]
fn should_reject_a_string_when_the_property_is_boolean() {
    let paths = Paths::para_test(std::env::temp_dir());
    let err = call(
        &paths,
        "leer",
        &json!({ "archivo": "test.PAN", "base64": "yes" }),
    )
    .expect_err("should reject string for boolean property");
    let msg = err.to_string();
    assert!(
        msg.contains("Wrong type for 'base64'")
            && msg.contains("expected boolean")
            && msg.contains("got string"),
        "error must name the property, expected type and received type: {msg}"
    );
}

/// `tools/list` and the boundary check agree on what `menu` and `uia_invocar` need.
#[test]
fn should_require_an_id_or_a_menu_and_item_for_the_menu_action() {
    let paths = Paths::para_test(std::env::temp_dir());
    let err = call(
        &paths,
        "gui_actuar",
        &json!({ "accion": "menu", "ventana": "THomeForm" }),
    )
    .expect_err("menu without id or position");
    let msg = err.to_string();
    assert!(
        msg.contains("id") && msg.contains("menu") && msg.contains("item"),
        "{msg}"
    );

    let err = call(
        &paths,
        "gui_actuar",
        &json!({ "accion": "uia_invocar", "ventana": "THomeForm" }),
    )
    .expect_err("uia_invocar without operacion");
    assert!(err.to_string().contains("operacion"), "{err}");
}
