use super::malla_valida;
use crate::error::Error;
use std::path::Path;

#[test]
fn should_reject_the_path_when_it_is_under_program_files() {
    let ruta = r"C:\Program Files\PVsyst8.1.8\malla.pvc";

    assert!(matches!(
        malla_valida(Path::new("."), ruta),
        Err(Error::Invalid(_))
    ));
}

/// GUI-05: a relative name was looked up in the process's folder, never in the workspace.
#[test]
fn should_look_up_a_relative_mesh_in_the_workspace() {
    let workspace = tempfile::tempdir().expect("temporary workspace");
    std::fs::write(workspace.path().join("malla.pvc"), b"x").expect("mesh file");

    let ruta = malla_valida(workspace.path(), "malla.pvc").expect("the mesh is in the workspace");

    assert!(ruta.ends_with("malla.pvc"));
    assert!(ruta.starts_with(workspace.path()), "{}", ruta.display());
}
