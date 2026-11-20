use super::parse_escena_texto;
use crate::error::Error;
use crate::escena_3d::dominio::TipoArchivoEscena;

const CASA_A: &str = include_str!("../../fixtures/escena_3d/Casa_A_8_1_8.SHD");

#[test]
fn should_reject_the_scene_when_the_file_is_cut_off_while_pvsyst_writes_it() {
    let corte = CASA_A
        .find("      PVObject_1=pvShdElemObject")
        .expect("objeto")
        + 120;

    let error = parse_escena_texto(&CASA_A.as_bytes()[..corte], TipoArchivoEscena::Shd)
        .expect_err("truncada");

    assert!(
        matches!(&error, Error::Format(m) if m.contains("truncada")),
        "{error}"
    );
}
