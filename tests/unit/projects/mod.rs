use super::*;

#[test]
fn should_reject_escape_when_confinement_is_workspace() {
    let ws = std::env::temp_dir();
    let res = resolve_project_file_con_alcance(&ws, "../../secret.PRJ", Alcance::Workspace);
    assert!(matches!(res, Err(crate::error::Error::Invalid(_))));
}

/// `\dir\X.PRJ` written without its drive letter: rooted, not absolute on Windows.
fn sin_unidad(p: &Path) -> String {
    let texto = p.to_string_lossy().into_owned();
    texto
        .split_once(':')
        .map_or(texto.clone(), |(_, resto)| resto.to_string())
}

#[test]
fn should_reject_rooted_name_without_drive_when_confinement_is_workspace() {
    let ws = tempfile::tempdir().expect("ws");
    std::fs::create_dir_all(ws.path().join("Projects")).expect("Projects");
    let fuera = tempfile::tempdir().expect("fuera");
    let prj = fuera.path().join("Ajeno.PRJ");
    std::fs::write(&prj, "PVObject_=pvProject\nEnd of PVObject pvProject\n").expect("prj");

    let res = resolve_project_file_con_alcance(ws.path(), &sin_unidad(&prj), Alcance::Workspace);

    assert!(
        matches!(&res, Err(Error::Invalid(m)) if m.contains("fuera del workspace")),
        "{res:?}"
    );
}
