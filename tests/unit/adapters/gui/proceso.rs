use super::*;
use std::sync::Arc;

#[test]
fn should_reject_project_outside_workspace_by_default() {
    let ws = std::env::temp_dir().join("test_pvsyst_prj_confined");
    let paths = crate::config::Paths {
        workspace: ws,
        cli_exe: PathBuf::new(),
        gui_exe: PathBuf::new(),
        aproximaciones: false,
        estado: Arc::default(),
    };
    let args = json!({});
    let res = resolver_proyecto(&paths, &args, "../../secret.PRJ");
    assert!(res.is_err(), "Must reject escaping project path by default");
}
