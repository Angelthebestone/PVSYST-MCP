use super::*;

#[test]
fn should_refuse_proyecto_when_pvsyst_is_already_running() {
    let err = comprobar_proyecto_libre(&json!({"proyecto": "x.PRJ"}), crate::idioma::Idioma::En)
        .unwrap_err()
        .to_string();
    assert!(err.contains("already running"), "{err}");
    assert!(err.contains("cerrar_todo"), "{err}");
}

#[test]
fn should_reject_project_outside_workspace_by_default() {
    let ws = std::env::temp_dir().join("test_pvsyst_prj_confined");
    let paths = crate::config::Paths::para_test(ws);
    let args = json!({});
    let res = resolver_proyecto(&paths, &args, "../../secret.PRJ");
    assert!(
        matches!(res, Err(crate::error::Error::Invalid(_))),
        "Must reject escaping project path by default"
    );
}
