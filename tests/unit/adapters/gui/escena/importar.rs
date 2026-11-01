use super::{en_carpeta_protegida, malla_valida, modales_de_importacion};
use crate::error::Error;
use std::path::Path;

fn malla_en_disco(nombre: &str) -> (tempfile::TempDir, String) {
    let carpeta = tempfile::tempdir().expect("carpeta temporal");
    let ruta = carpeta.path().join(nombre);
    std::fs::write(&ruta, b"malla").expect("archivo de prueba");
    let texto = ruta.display().to_string();
    (carpeta, texto)
}

#[test]
fn should_accept_the_mesh_when_it_exists_with_a_known_extension() {
    for nombre in ["a.pvc", "b.PVC2", "c.3ds", "d.dae", "e.pvz2"] {
        let (_carpeta, ruta) = malla_en_disco(nombre);

        assert!(malla_valida(&ruta).is_ok(), "{nombre}");
    }
}

#[test]
fn should_reject_the_file_when_the_extension_is_not_a_mesh() {
    let (_carpeta, ruta) = malla_en_disco("escena.obj");

    assert!(matches!(malla_valida(&ruta), Err(Error::Invalid(_))));
}

#[test]
fn should_reject_the_path_when_it_is_under_program_files() {
    let ruta = r"C:\Program Files\PVsyst8.1.8\malla.pvc";

    assert!(matches!(malla_valida(ruta), Err(Error::Invalid(_))));
}

#[test]
fn should_report_not_found_when_the_mesh_does_not_exist() {
    let (carpeta, _) = malla_en_disco("otra.pvc");
    let ausente = carpeta.path().join("ausente.pvc").display().to_string();

    assert!(matches!(malla_valida(&ausente), Err(Error::NotFound(_))));
}

#[test]
fn should_accept_a_folder_that_only_contains_the_words_when_it_is_not_protected() {
    let ruta = Path::new(r"C:\Users\ana\my program files notes\mesh.3ds");

    assert!(!en_carpeta_protegida(ruta));
}

#[test]
fn should_flag_the_protected_folders_when_the_component_matches_in_any_case() {
    for ruta in [
        r"C:\Program Files\PVsyst8.1.8\malla.pvc",
        r"c:\PROGRAM FILES (X86)\x\malla.pvc",
        r"C:\PROGRA~1\PVsyst\malla.pvc",
        r"C:\progra~2\x\malla.pvc",
    ] {
        assert!(en_carpeta_protegida(Path::new(ruta)), "{ruta}");
    }
}

#[test]
fn should_not_flag_a_folder_when_the_short_alias_is_not_program_files() {
    for ruta in [r"C:\PROGRA~\x", r"C:\PROGRA~A\x", r"C:\PROGRAMA~1\x"] {
        assert!(!en_carpeta_protegida(Path::new(ruta)), "{ruta}");
    }
}

#[test]
fn should_flag_a_relative_name_when_it_resolves_inside_program_files() {
    let cwd = Path::new(r"C:\Program Files\PVsyst8.1.8");

    assert!(en_carpeta_protegida(&cwd.join("malla.pvc")));
}

#[test]
fn should_keep_the_modals_in_front_to_back_order_when_the_import_left_several_open() {
    let abiertas = [
        (1, "TMessageForm".to_string()),
        (2, "TProjectForm".to_string()),
        (3, "#32770".to_string()),
        (4, "TImportResultForm".to_string()),
    ];

    assert_eq!(modales_de_importacion(&abiertas), vec![1, 3, 4]);
}

#[test]
fn should_close_nothing_when_no_import_modal_is_open() {
    let abiertas = [(2, "TProjectForm".to_string())];

    assert!(modales_de_importacion(&abiertas).is_empty());
}

#[test]
fn should_accept_the_mesh_when_its_folder_name_only_contains_program_files() {
    let carpeta = tempfile::Builder::new()
        .prefix("my program files notes")
        .tempdir()
        .expect("carpeta temporal");
    let ruta = carpeta.path().join("mesh.3ds");
    std::fs::write(&ruta, b"malla").expect("archivo de prueba");

    assert!(malla_valida(&ruta.display().to_string()).is_ok());
}
