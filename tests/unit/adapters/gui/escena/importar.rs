use super::malla_valida;
use crate::error::Error;

#[test]
fn should_reject_the_path_when_it_is_under_program_files() {
    let ruta = r"C:\Program Files\PVsyst8.1.8\malla.pvc";

    assert!(matches!(malla_valida(ruta), Err(Error::Invalid(_))));
}
