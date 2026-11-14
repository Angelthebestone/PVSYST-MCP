use super::*;
use crate::prefs::ini::parsear;
use std::path::Path;

#[test]
fn should_accept_any_workspace_when_the_cli_is_not_the_real_one() {
    let paths = Paths::para_test("C:/cualquiera");
    assert!(exigir(&paths, Path::new("C:/otro"), "create-site").is_ok());
}

#[test]
fn should_tell_the_real_cli_from_the_fake_one_by_its_file_name() {
    let mut paths = Paths::para_test("C:/ws");
    paths.cli_exe = "C:/Program Files/PVsyst8.1.8/pvsystcli.EXE".into();
    assert!(es_cli_real(&paths));
    paths.cli_exe = "C:/target/debug/fake_pvsystcli.exe".into();
    assert!(!es_cli_real(&paths));
}

#[test]
fn should_read_datapath_from_the_system_section_ignoring_case() {
    let secciones = parsear("[system]\r\ndatapath=C:\\Datos\r\n");
    assert_eq!(
        valor_ini(&secciones, "System", "DataPath").as_deref(),
        Some("C:\\Datos")
    );
    assert_eq!(valor_ini(&secciones, "System", "Otra"), None);
}

#[test]
fn should_match_workspaces_that_differ_in_slashes_case_or_trailing_separator() {
    assert!(mismas_rutas(
        Path::new("C:/No/Existe/Data/"),
        Path::new("c:\\no\\existe\\data")
    ));
    assert!(!mismas_rutas(
        Path::new("C:/No/Existe/Data"),
        Path::new("C:/No/Existe/Otro")
    ));
}
