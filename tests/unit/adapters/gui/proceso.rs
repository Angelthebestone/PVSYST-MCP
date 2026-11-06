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
    assert!(
        matches!(res, Err(crate::error::Error::Invalid(_))),
        "Must reject escaping project path by default"
    );
}

#[test]
fn should_read_the_windows_again_when_a_close_reopened_a_parent() {
    assert!(otra_ronda(2, 0));
    assert!(otra_ronda(1, MAX_RONDAS_CIERRE - 1));
}

#[test]
fn should_stop_when_no_window_is_left() {
    assert!(!otra_ronda(0, 0));
}

#[test]
fn should_stop_after_the_last_round_even_if_windows_remain() {
    assert!(!otra_ronda(1, MAX_RONDAS_CIERRE));
}
