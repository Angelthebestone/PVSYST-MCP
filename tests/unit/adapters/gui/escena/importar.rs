use super::{en_carpeta_protegida, malla_valida};
use crate::error::Error;
use std::path::Path;

#[test]
fn should_reject_the_path_when_it_is_under_program_files() {
    let ruta = r"C:\Program Files\PVsyst8.1.8\malla.pvc";

    assert!(matches!(malla_valida(ruta), Err(Error::Invalid(_))));
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
