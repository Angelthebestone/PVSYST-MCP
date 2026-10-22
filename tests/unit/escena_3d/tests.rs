use super::aplicacion::leer_escena;
use super::codec::parse_escena_3d;
use super::dominio::TipoArchivoEscena;
use crate::config::Paths;
use serde_json::json;
use std::fs;
use std::path::{Path, PathBuf};

const FIXTURE_ESCENA_SINTETICA: &[u8] =
    b"pvShdBuilding;Synthetic Test;\r\nVersion 4.21; Calcul=$0001;\r\npvShdElemObject;Box;\r\nVersion 4.21; Calcul=$0005;\r\n";

fn paths_de_test(ws: &Path) -> Paths {
    Paths {
        workspace: ws.to_path_buf(),
        cli_exe: PathBuf::new(),
        gui_exe: PathBuf::new(),
        aproximaciones: Default::default(),
        estado: Default::default(),
    }
}

#[test]
fn should_parse_golden_synthetic_3d_scene() {
    let escena = parse_escena_3d(FIXTURE_ESCENA_SINTETICA, TipoArchivoEscena::Bld)
        .expect("Parseo de escena sintética");

    assert_eq!(escena.clase_raiz(), "pvShdBuilding");
    assert_eq!(escena.descripcion(), "Synthetic Test");
    assert_eq!(escena.objetos().len(), 1);
    assert_eq!(escena.objetos()[0].clase(), "pvShdElemObject");
    assert_eq!(escena.objetos()[0].nombre(), "Box");
}

#[test]
fn should_reject_unsupported_3d_file_extension() {
    let err = TipoArchivoEscena::de_extension("XYZ");
    assert!(err.is_err());
}

#[test]
fn should_reject_invalid_root_class_in_3d_scene() {
    let raw = b"pvInvalido;Desc;\r\nVersion 4.0; Calcul=$00;\r\n";
    let err = parse_escena_3d(raw, TipoArchivoEscena::Shd);
    assert!(err.is_err());
}

#[test]
fn should_reject_mismatched_extension_and_root_class() {
    let raw = b"pvShading;Desc;\r\nVersion 4.0; Calcul=$00;\r\n";
    let err = parse_escena_3d(raw, TipoArchivoEscena::Bld);
    assert!(err.is_err());
}

#[test]
fn should_reject_invalid_root_version_line() {
    let raw = b"pvShading;Desc;\r\nbasura;\r\n";
    let err = parse_escena_3d(raw, TipoArchivoEscena::Shd);
    assert!(err.is_err());
}

#[test]
fn should_parse_synthetic_3d_scene_with_binary_position_and_geometry() {
    use crate::format::real48::encode_real48;
    let mut sample = b"pvShdBuilding;Test;\r\nVersion 4.21; Calcul=$0001;\r\npvShdElemObject;Box;\r\nVersion 4.21; Calcul=$0005;\r\n".to_vec();
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
fn should_read_scene_via_observable_tool_json() {
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

#[test]
fn should_reject_scene_without_extension() {
    let ws = tempfile::tempdir().expect("Directorio temporal");
    let shadings_dir = ws.path().join("Shadings");
    fs::create_dir_all(&shadings_dir).expect("Crear Shadings");
    fs::write(shadings_dir.join("SinExtension"), FIXTURE_ESCENA_SINTETICA).expect("Escritura");
    let paths = paths_de_test(ws.path());
    let err = leer_escena(&paths, &json!({ "archivo": "SinExtension" }));
    assert!(err.is_err());
    assert!(err.unwrap_err().to_string().contains("extensión"));
}

#[test]
fn should_reject_path_traversal_when_reading_scene() {
    let ws = tempfile::tempdir().expect("Directorio temporal");
    let paths = paths_de_test(ws.path());
    let err = leer_escena(&paths, &json!({ "archivo": "../../Escena.SHD" }));
    assert!(err.is_err());
}

#[test]
fn should_reject_external_absolute_path_without_permission() {
    let ws = tempfile::tempdir().expect("Directorio temporal");
    let paths = paths_de_test(ws.path());
    let ruta_externa = if cfg!(windows) {
        r"C:\Windows\System32\drivers\etc\hosts"
    } else {
        "/etc/hosts"
    };
    let err = leer_escena(&paths, &json!({ "archivo": ruta_externa }));
    assert!(err.is_err());
}

#[test]
fn should_reject_truncated_3d_file_without_panicking() {
    assert!(parse_escena_3d(b"", TipoArchivoEscena::Shd).is_err());
    assert!(parse_escena_3d(b"pvShading", TipoArchivoEscena::Shd).is_err());
    assert!(parse_escena_3d(b"pvShading;Desc;\r\n", TipoArchivoEscena::Shd).is_err());
    let cortado = b"pvShading;Desc;\r\nVersion 4.0; Calcul=$0000;\r\npvShdElemObject;Box;\r\n";
    let res = parse_escena_3d(cortado, TipoArchivoEscena::Shd);
    assert!(res.is_ok());
    let mut con_bloque_incompleto = cortado.to_vec();
    con_bloque_incompleto.extend_from_slice(&[0u8; 10]);
    assert!(parse_escena_3d(&con_bloque_incompleto, TipoArchivoEscena::Shd).is_ok());
}

#[test]
#[ignore = "necesita la instalación de PVsyst"]
fn should_decode_real_demo_geneve_grid_bld_sample() {
    let ruta = PathBuf::from(
        r"C:\Program Files\PVsyst8.1.7\DataRO\PVsyst8.1_Data\Shadings\DEMO_Geneve_Grid.BLD",
    );
    if !ruta.exists() {
        panic!(
            "Falta el archivo requerido de la instalación de PVsyst: {}",
            ruta.display()
        );
    }
    let bytes = fs::read(&ruta).expect("Lectura de DEMO_Geneve_Grid.BLD");
    let escena = parse_escena_3d(&bytes, TipoArchivoEscena::Bld).expect("Parseo de BLD real");

    assert_eq!(escena.clase_raiz(), "pvShdBuilding");
    assert_eq!(escena.objetos().len(), 3);
    assert_eq!(escena.objetos()[0].clase(), "pvShdElemObject");
    assert_eq!(escena.objetos()[0].nombre(), "Parallélépipède");
    let pos = escena.objetos()[0].posicion().expect("posicion");
    assert!((pos.x - (-5.0)).abs() < 1e-3);
    let geo = escena.objetos()[0].geometria().expect("geometria");
    assert_eq!(geo.forma.nombre(), "Parallélépipède");
    assert_eq!(escena.objetos()[1].nombre(), "Toiture + pignons");
}
