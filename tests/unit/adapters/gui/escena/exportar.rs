//! File policy of a scene export: PVsyst writes a temporary name, a read deletes
//! it and a write replaces the requested name.

use super::{borrar, conservar, preparar_temporal, ruta_de, ruta_temporal};
use crate::config::Paths;

fn workspace() -> (tempfile::TempDir, Paths) {
    let carpeta = tempfile::tempdir().expect("carpeta temporal");
    let paths = Paths::para_test(carpeta.path());
    std::fs::create_dir_all(carpeta.path().join("Shadings")).expect("carpeta Shadings");
    (carpeta, paths)
}

#[test]
fn should_use_a_free_name_when_the_scene_file_already_exists() {
    let (_carpeta, paths) = workspace();
    let definitiva = ruta_de(&paths, "principal");
    std::fs::write(&definitiva, b"escena anterior").expect("escena previa");

    let temporal = ruta_temporal(&paths, "principal");

    assert_ne!(temporal, definitiva);
    assert!(!temporal.exists(), "{}", temporal.display());
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

#[test]
fn should_reuse_its_own_shd_export_on_repeated_escena_leer() {
    let (_carpeta, paths) = workspace();
    for _ in 0..2 {
        let temporal = preparar_temporal(&paths, "principal");
        std::fs::write(&temporal, b"lectura").expect("exportacion de lectura");
        borrar(&temporal);
        assert!(!temporal.exists(), "{}", temporal.display());
    }
    assert!(!ruta_de(&paths, "principal").exists());
}

#[test]
fn should_not_block_escena_write_after_escena_leer() {
    let (_carpeta, paths) = workspace();
    let lectura = preparar_temporal(&paths, "principal");
    std::fs::write(&lectura, b"lectura").expect("exportacion de lectura");
    borrar(&lectura);

    let escritura = preparar_temporal(&paths, "principal");
    std::fs::write(&escritura, b"escena escrita").expect("escena escrita");
    let guardada = conservar(&escritura, &ruta_de(&paths, "principal")).expect("se conserva");

    assert_eq!(std::fs::read(guardada).expect("escena"), b"escena escrita");
}

#[test]
fn should_remove_a_leftover_export_before_writing_again() {
    let (_carpeta, paths) = workspace();
    let temporal = ruta_temporal(&paths, "principal");
    std::fs::write(&temporal, b"resto de una llamada interrumpida").expect("resto");

    let nueva = preparar_temporal(&paths, "principal");

    assert!(!nueva.exists(), "{}", nueva.display());
}
