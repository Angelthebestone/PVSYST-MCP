//! Unit and behavioral tests for MCP tool declarations, validation and dispatch.

use super::*;
use crate::config::Paths;
use serde_json::json;

#[test]
fn should_reject_missing_required_arg() {
    let paths = Paths::resolve();
    // 'leer' requires 'archivo'
    let err = call(&paths, "leer", &json!({})).expect_err("should reject missing required arg");
    let msg = err.to_string();
    assert!(
        msg.contains("Missing required argument 'archivo'"),
        "error message must name the missing field: {msg}"
    );
}

#[test]
fn should_reject_wrong_type_arg_mentioning_tipo() {
    let paths = Paths::resolve();
    // 'base64' is declared boolean; passing a string "yes" must fail at boundary
    let err = call(
        &paths,
        "leer",
        &json!({ "archivo": "test.PAN", "base64": "yes" }),
    )
    .expect_err("should reject string for boolean property");
    let msg = err.to_string();
    assert!(
        msg.to_lowercase().contains("tipo"),
        "error message must mention 'tipo': {msg}"
    );
    assert!(
        msg.contains("base64"),
        "error message must name the invalid field: {msg}"
    );
    assert!(
        msg.contains("esperado boolean") && msg.contains("recibido string"),
        "error message must specify expected and received types: {msg}"
    );
}

#[test]
fn should_reject_wrong_type_for_numeric_property() {
    let paths = Paths::resolve();
    // 'n_inversores' in 'proyecto' is integer; passing a boolean must fail
    let err = call(
        &paths,
        "proyecto",
        &json!({ "accion": "crear", "plantilla": "DEMO", "nombre_nuevo": "NUEVO", "n_inversores": true }),
    )
    .expect_err("should reject boolean for integer property");
    let msg = err.to_string();
    assert!(
        msg.to_lowercase().contains("tipo"),
        "error must mention 'tipo': {msg}"
    );
    assert!(msg.contains("n_inversores"), "error must name field: {msg}");
}

#[test]
fn should_reject_missing_discriminator_when_required() {
    let paths = Paths::resolve();
    // 'proyecto' requires 'accion' (no default)
    let err = call(&paths, "proyecto", &json!({})).expect_err("should require accion");
    let msg = err.to_string();
    assert!(
        msg.contains("Missing required argument 'accion'"),
        "should name the missing discriminator: {msg}"
    );
}

#[test]
fn should_apply_default_discriminator_when_omitted() {
    let paths = Paths::resolve();
    // 'entorno' defaults 'accion' to 'diagnostico'
    let res = call(&paths, "entorno", &json!({})).expect("should succeed with default diagnostico");
    assert!(res.get("entorno").is_some());
}

#[test]
fn should_reject_invalid_discriminator_value() {
    let paths = Paths::resolve();
    let err = call(&paths, "entorno", &json!({ "accion": "inexistente" }))
        .expect_err("should reject invalid accion");
    let msg = err.to_string();
    assert!(
        msg.contains("Valor 'inexistente' no válido"),
        "should reject invalid value: {msg}"
    );
}

#[test]
fn should_reject_wrong_type_for_discriminator() {
    let paths = Paths::resolve();
    let err = call(&paths, "entorno", &json!({ "accion": 12345 }))
        .expect_err("should reject number for accion");
    let msg = err.to_string();
    assert!(
        msg.to_lowercase().contains("tipo"),
        "should mention tipo: {msg}"
    );
    assert!(msg.contains("accion"), "should mention field name: {msg}");
}

#[test]
fn should_accept_compatible_union_types() {
    // 'n_mod_serie' is integer|string
    let h = REGISTRO.iter().find(|h| h.nombre == "proyecto").unwrap();
    let val_int =
        json!({ "accion": "crear", "plantilla": "P", "nombre_nuevo": "N", "n_mod_serie": 20 });
    let val_str =
        json!({ "accion": "crear", "plantilla": "P", "nombre_nuevo": "N", "n_mod_serie": "20" });
    assert!(h.validar(&val_int).is_ok());
    assert!(h.validar(&val_str).is_ok());

    let val_bad =
        json!({ "accion": "crear", "plantilla": "P", "nombre_nuevo": "N", "n_mod_serie": false });
    assert!(h.validar(&val_bad).is_err());
}

#[test]
fn should_enforce_expert_tool_permission() {
    let mut paths = Paths::resolve();
    paths.experto = false;
    let err = call(&paths, "pvsyst_cli", &json!({ "comando": "version" }))
        .expect_err("expert tool must be blocked without expert flag");
    assert!(err.to_string().contains("Herramienta experta"));

    paths.experto = true;
    // With expert flag, validation runs (will fail dry_run or missing command if invalid)
    let schemas = list(&paths);
    assert_eq!(schemas.len(), 15);
}

#[test]
fn should_include_permitir_fuera_del_workspace_property() {
    let paths = Paths::resolve();
    let schemas = list(&paths);
    for tool_name in [
        "leer",
        "listar",
        "editar",
        "escribir_archivo",
        "componente",
        "sitio",
        "resultados",
        "workspace",
    ] {
        let tool_schema = schemas
            .iter()
            .find(|s| s["name"] == tool_name)
            .unwrap_or_else(|| panic!("missing tool {tool_name}"));
        assert!(
            tool_schema["inputSchema"]["properties"]
                .get("permitir_fuera_del_workspace")
                .is_some(),
            "tool {tool_name} must declare permitir_fuera_del_workspace in its properties"
        );
    }
}
