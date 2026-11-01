use super::parse_escena_texto;
use crate::error::Error;
use crate::escena_3d::dominio::TipoArchivoEscena;

const CASA_A: &str = include_str!("../../fixtures/escena_3d/Casa_A_8_1_8.SHD");

fn caja(indice: u32, nombre: &str) -> String {
    format!(
        "          PVObject_{indice}=pvShdElemObject\n            Comment={nombre}\n            Version=8.1.8\n            Flags=$02200004\n            DOrigine=0.000000,0.000000,0.000000\n            DInclin=0.000000\n            DAzimut=0.000000\n            ObjetElemType=oeParallelepipede\n            Params=1.000,1.000,1.000,\n          End of PVObject pvShdElemObject\n"
    )
}

fn escena_con_grupo() -> String {
    format!(
        "PVObject_=pvShading\n    Comment=Grupo\n    Version=8.1.8\n    Flags=$00\n\n    ListeObjets, list of=1 TShdObject\n\n      PVObject_1=pvShdObjectGroup\n        Comment=Casas\n        ListeObjets, list of=2 TShdObject\n{}{}        End of ListeObjets\n      End of PVObject pvShdObjectGroup\n    End of ListeObjets\n  End of PVObject pvShading\n",
        caja(1, "A"),
        caja(2, "B")
    )
}

#[test]
fn should_count_every_object_when_they_are_grouped_in_a_container() {
    let escena = parse_escena_texto(escena_con_grupo().as_bytes(), TipoArchivoEscena::Shd)
        .expect("escena agrupada");

    let nombres: Vec<&str> = escena.objetos().iter().map(|o| o.nombre()).collect();

    assert_eq!(nombres, ["A", "B"]);
}

#[test]
fn should_count_a_container_as_one_object_when_nothing_is_inside_it() {
    let texto = "PVObject_=pvShading\n    Comment=Vacio\n    Version=8.1.8\n    Flags=$00\n\n    ListeObjets, list of=1 TShdObject\n\n      PVObject_1=pvShdObjectGroup\n        Comment=Hueco\n      End of PVObject pvShdObjectGroup\n    End of ListeObjets\n  End of PVObject pvShading\n";

    let escena = parse_escena_texto(texto.as_bytes(), TipoArchivoEscena::Shd).expect("escena");

    assert_eq!(escena.objetos().len(), 1);
}

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

#[test]
fn should_read_the_saved_scene_when_it_is_complete() {
    let escena = parse_escena_texto(CASA_A.as_bytes(), TipoArchivoEscena::Shd).expect("completa");

    assert_eq!(escena.objetos().len(), 1);
}
