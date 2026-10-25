use super::*;

#[test]
fn should_refuse_proyecto_when_pvsyst_is_already_running() {
    let err = comprobar_proyecto_libre(&json!({"proyecto": "x.PRJ"}))
        .unwrap_err()
        .to_string();
    assert!(err.contains("ya está en ejecución"), "{err}");
    assert!(err.contains("cerrar_todo"), "{err}");
}

#[test]
fn should_allow_a_launch_without_proyecto_while_running() {
    assert!(comprobar_proyecto_libre(&json!({})).is_ok());
}

#[test]
fn should_reject_project_outside_workspace_by_default() {
    let ws = std::env::temp_dir().join("test_pvsyst_prj_confined");
    let paths = crate::config::Paths::para_test(ws);
    let args = json!({});
    let res = resolver_proyecto(&paths, &args, "../../secret.PRJ");
    assert!(res.is_err(), "Must reject escaping project path by default");
}
