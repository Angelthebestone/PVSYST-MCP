use super::*;

#[test]
fn should_not_probe_paths_outside_projects_when_name_is_a_path() {
    let ws = tempfile::tempdir().expect("ws");
    let fuera = tempfile::tempdir().expect("fuera");
    std::fs::create_dir_all(ws.path().join("Projects")).expect("Projects");
    std::fs::write(fuera.path().join("Ajeno.PRJ"), "x").expect("prj");
    let ajeno = fuera.path().join("Ajeno").to_string_lossy().into_owned();

    assert!(!es_proyecto(ws.path(), &ajeno, ""));
    assert!(!es_proyecto(ws.path(), "..\\..\\Ajeno", ""));
}
