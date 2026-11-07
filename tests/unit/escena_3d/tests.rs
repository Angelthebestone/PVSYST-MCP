use super::aplicacion::leer_escena;
use super::codec::parse_escena_3d;
use super::dominio::TipoArchivoEscena;
use crate::config::Paths;
use serde_json::json;
use std::fs;
use std::path::Path;

const FIXTURE_ESCENA_SINTETICA: &[u8] =
    b"pvShdBuilding;Synthetic Test;\r\nVersion 4.21; Calcul=$0001;\r\npvShdElemObject;Box;\r\nVersion 4.21; Calcul=$0005;\r\n";

/// Scene saved by PVsyst 8.1.8 (PRUEBA) from the editor: one box, 10 x 8 x 3 m.
const ESCENA_TEXTO_8_1_8: &[u8] = include_bytes!("../../fixtures/escena_3d/Casa_A_8_1_8.SHD");

fn paths_de_test(ws: &Path) -> Paths {
    Paths::para_test(ws)
}

#[test]
fn should_parse_golden_scene_when_the_root_class_matches_the_extension() {
    let escena = parse_escena_3d(FIXTURE_ESCENA_SINTETICA, TipoArchivoEscena::Bld)
        .expect("Parseo de escena sintética");

    assert_eq!(escena.clase_raiz(), "pvShdBuilding");
    assert_eq!(escena.descripcion(), "Synthetic Test");
    assert_eq!(escena.objetos().len(), 1);
    assert_eq!(escena.objetos()[0].clase(), "pvShdElemObject");
    assert_eq!(escena.objetos()[0].nombre(), "Box");

    let cruzado = b"pvShading;Desc;\r\nVersion 4.0; Calcul=$00;\r\n";
    assert!(parse_escena_3d(cruzado, TipoArchivoEscena::Bld).is_err());
}

#[test]
fn should_read_the_box_when_the_scene_is_the_text_form_with_a_byte_order_mark() {
    let escena = parse_escena_3d(ESCENA_TEXTO_8_1_8, TipoArchivoEscena::Shd).expect("texto");

    assert_eq!(escena.clase_raiz(), "pvShading");
    assert_eq!(escena.version(), "8.1.8");
    assert_eq!(escena.objetos().len(), 1);
    let caja = &escena.objetos()[0];
    assert_eq!((caja.clase(), caja.nombre()), ("pvShdElemObject", "Casa_A"));
    let geo = caja.geometria().expect("geometria");
    assert_eq!(geo.tipo_pvsyst.as_deref(), Some("oeParallelepipede"));
    assert_eq!(geo.dimensiones, vec![10.0, 8.0, 3.0]);
    assert_eq!(caja.posicion().expect("posicion").z, 0.0);
}

#[test]
fn should_read_scene_through_the_tool_json_when_the_file_is_in_shadings() {
    let ws = tempfile::tempdir().expect("Directorio temporal");
    let shadings_dir = ws.path().join("Shadings");
    fs::create_dir_all(&shadings_dir).expect("Crear Shadings");
    fs::write(shadings_dir.join("Edificio.BLD"), FIXTURE_ESCENA_SINTETICA).expect("Escritura");

    let paths = paths_de_test(ws.path());
    let leida =
        leer_escena(&paths, &json!({ "archivo": "Edificio.BLD" })).expect("Lectura de escena");

    assert_eq!(leida["tipo_archivo"], "BLD");
    assert_eq!(leida["clase_raiz"], "pvShdBuilding");
    assert_eq!(leida["total_objetos"], 1);
    assert_eq!(leida["objetos"][0]["nombre"], "Box");
}
