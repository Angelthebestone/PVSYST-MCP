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
