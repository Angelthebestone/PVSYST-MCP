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

#[test]
fn should_validate_every_configured_step_before_applying_the_first() {
    let ws = tempfile::tempdir().expect("ws");
    let paths = Paths::para_test(ws.path());
    let args = json!({
        "accion": "configurar",
        "proyecto": "Planta",
        "pasos": [
            { "accion": "perdidas", "suciedad_pct": 2 },
            { "accion": "limite_inyeccion" }
        ]
    });

    let err = call(&paths, "proyecto", &args).expect_err("paso 2 sin potencia_kw");

    let msg = err.to_string();
    assert!(msg.contains("potencia_kw"), "{msg}");
    assert!(!msg.contains("Paso 1 fallido"), "{msg}");
}

#[test]
fn should_keep_the_error_kind_of_a_failed_step() {
    let ws = tempfile::tempdir().expect("ws");
    let paths = Paths::para_test(ws.path());
    let args = json!({ "accion": "configurar", "proyecto": "NoExiste", "pasos": [ { "accion": "perdidas", "suciedad_pct": 2 } ] });

    let err = call(&paths, "proyecto", &args).expect_err("proyecto inexistente");

    assert!(
        matches!(&err, crate::error::Error::NotFound(m) if m.starts_with("Step 1 failed")),
        "{err:?}"
    );
}
