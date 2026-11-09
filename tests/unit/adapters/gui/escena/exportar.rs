//! File policy of a scene export: PVsyst writes a temporary name, a read deletes
//! it and a write replaces the requested name.

use super::{borrar, conservar, exigir_nombre_escrito, ruta_de, ruta_temporal};
use crate::config::Paths;

/// GUI-03: «Guardar» with a name that never reached the field wrote nothing and said nothing.
#[test]
fn should_refuse_to_save_when_the_name_did_not_reach_the_dialog() {
    let error = exigir_nombre_escrito("_mcp_QA", "").expect_err("empty field");

    assert!(error.to_string().contains("_mcp_QA"), "{error}");
    assert!(exigir_nombre_escrito("_mcp_QA", "_mcp_QA").is_ok());
}

fn workspace() -> (tempfile::TempDir, Paths) {
    let carpeta = tempfile::tempdir().expect("carpeta temporal");
    let paths = Paths::para_test(carpeta.path());
    std::fs::create_dir_all(carpeta.path().join("Shadings")).expect("carpeta Shadings");
    (carpeta, paths)
}

#[test]
fn should_replace_its_own_export_when_the_scene_is_written_again() {
    let (_carpeta, paths) = workspace();
    let definitiva = ruta_de(&paths, "principal");
    std::fs::write(&definitiva, b"escena anterior").expect("escena previa");
    let temporal = ruta_temporal(&paths, "principal");
    std::fs::write(&temporal, b"escena nueva").expect("escena nueva");

    let guardada = conservar(&temporal, &definitiva).expect("se conserva la escena");

    assert_eq!(guardada, definitiva);
    assert_eq!(
        std::fs::read(&definitiva).expect("escena guardada"),
        b"escena nueva"
    );
    assert!(!temporal.exists());
}

#[test]
fn should_leave_the_scene_file_untouched_when_a_read_deletes_its_export() {
    let (_carpeta, paths) = workspace();
    let definitiva = ruta_de(&paths, "principal");
    std::fs::write(&definitiva, b"escena del usuario").expect("escena previa");
    let temporal = ruta_temporal(&paths, "principal");
    std::fs::write(&temporal, b"lectura").expect("exportacion de lectura");

    borrar(&temporal);

    assert!(!temporal.exists());
    assert_eq!(
        std::fs::read(&definitiva).expect("escena intacta"),
        b"escena del usuario"
    );
}
