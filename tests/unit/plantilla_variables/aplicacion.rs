use super::*;
use tempfile::TempDir;

fn workspace_temporal() -> (TempDir, Paths) {
    let tmp = TempDir::new().expect("temp dir");
    let paths = Paths::para_test(tmp.path());
    (tmp, paths)
}

#[test]
fn should_create_and_list_rvt_file() {
    let (_tmp, paths) = workspace_temporal();
    let rvt_nombre = "TestVariables.RVT";
    let args_crear = json!({
        "archivo": rvt_nombre,
        "operacion": "crear",
        "variables": ["HSol", "AzSol", "E_Grid"]
    });
    let res_crear = gestionar_rvt(&paths, &args_crear).expect("crear");
    assert_eq!(res_crear["ok"], true);
    assert_eq!(res_crear["total"], 3);

    let args_listar = json!({
        "archivo": rvt_nombre,
        "operacion": "listar"
    });
    let res_listar = gestionar_rvt(&paths, &args_listar).expect("listar");
    assert_eq!(res_listar["total"], 3);
    assert_eq!(res_listar["variables"], json!(["HSol", "AzSol", "E_Grid"]));
}

#[test]
fn should_reject_rvt_outside_workspace_when_not_permitted() {
    let (_tmp, paths) = workspace_temporal();
    let fuera = TempDir::new().expect("fuera");
    let ajeno = fuera.path().join("Ajeno.RVT");
    for archivo in [
        ajeno.to_string_lossy().into_owned(),
        "../Ajeno.RVT".to_string(),
    ] {
        let args = json!({ "archivo": archivo, "operacion": "crear", "variables": ["HSol"] });
        let res = gestionar_rvt(&paths, &args);
        assert!(matches!(res, Err(Error::Invalid(_))), "{archivo}: {res:?}");
    }
    assert!(!ajeno.exists());
}

#[test]
fn should_reject_variables_that_are_not_names() {
    let (_tmp, paths) = workspace_temporal();
    let args = json!({ "archivo": "X.RVT", "operacion": "crear", "variables": ["HSol", 7] });

    let res = gestionar_rvt(&paths, &args);

    assert!(matches!(res, Err(Error::Invalid(m)) if m.contains("variables")));
}
