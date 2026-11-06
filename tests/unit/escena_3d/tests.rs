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
fn should_parse_position_and_geometry_blocks_when_the_scene_carries_binary_objects() {
    use crate::format::real48::encode_real48;
    let mut sample = FIXTURE_ESCENA_SINTETICA.to_vec();
    // 44-byte position block: 12 bytes prefix, 5x Real48 slots (x, y, z, rot1, rot2), visibility, '/'
    let mut pos_block = vec![0u8; 12];
    pos_block.extend_from_slice(&encode_real48(10.5).unwrap());
    pos_block.extend_from_slice(&encode_real48(-2.0).unwrap());
    pos_block.extend_from_slice(&encode_real48(3.0).unwrap());
    pos_block.extend_from_slice(&encode_real48(45.0).unwrap());
    pos_block.extend_from_slice(&encode_real48(0.0).unwrap());
    pos_block.push(1); // visible
    pos_block.push(b'/');
    pos_block.extend_from_slice(b"\r\n");
    sample.extend_from_slice(&pos_block);

    // 39-byte geometry block: shape 11 (parallelepiped), 6x Real48 slots, '/'
    let mut geom_block = vec![11u8, 0u8];
    for dim in [5.0, 4.0, 3.0, 0.0, 0.0, 1.0] {
        geom_block.extend_from_slice(&encode_real48(dim).unwrap());
    }
    geom_block.push(b'/');
    geom_block.extend_from_slice(b"\r\n");
    sample.extend_from_slice(&geom_block);

    let escena = parse_escena_3d(&sample, TipoArchivoEscena::Bld).expect("parseo con bloques");
    let obj = &escena.objetos()[0];
    let pos = obj.posicion().expect("posicion presente");
    assert!((pos.x - 10.5).abs() < 1e-4);
    assert!((pos.y - (-2.0)).abs() < 1e-4);
    let geo = obj.geometria().expect("geometria presente");
    assert_eq!(geo.forma.nombre(), "Parallélépipède");
    assert!((geo.dimensiones[0] - 5.0).abs() < 1e-4);
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
