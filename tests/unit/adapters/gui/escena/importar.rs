use super::malla_valida;
use crate::error::Error;

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
