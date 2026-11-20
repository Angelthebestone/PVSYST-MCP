use super::*;

/// GUI-15: the project went to PVsyst's command line, which ignores it.
#[test]
fn should_launch_without_the_project_that_pvsyst_would_ignore() {
    let args = json!({ "accion": "lanzar", "proyecto": "QA_X", "cerrar_demo": true });

    assert!(pide_proyecto(&args));
    let sin = sin_proyecto(&args);

    assert!(sin.get("proyecto").is_none());
    assert_eq!(sin["cerrar_demo"], true);
    assert!(!pide_proyecto(&sin));
}
