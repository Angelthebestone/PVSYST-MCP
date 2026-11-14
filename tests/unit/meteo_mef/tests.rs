use super::aplicacion::{leer_mef, validar_mef};
use super::codec::{encode_mef, parse_mef};
use crate::config::Paths;
use serde_json::json;
use std::fs;

const FIXTURE_LEGACY_MEF: &[u8] = b"pvMeteoFormat;Sample Legacy Format;\r\nVersion 5.67; Calcul=$00003043;\r\nDelim=;\r\nGlobHor;Irrad;2;0;1500;1.0;\r\nT_Amb;Temper;3;-40;60;1.0;\r\nWindVel;WindVel;4;0;50;1.0;\r\n";

const FIXTURE_MODERNO_CON_DESCONOCIDO: &[u8] = b"\xef\xbb\xbfPVObject_=pvMeteoFormat\r\n  Comment=Modern Test\r\n  Version=8.1.0\r\n  DateFmtType=MM_DD_hh_mm\r\n  Separ=$003B\r\n  CampoDesconocido=ValorArbitrario\r\n  MeasureStep=60\r\n  AlbedoSolMF=0.20\r\n  NoFieldDate=1,2,0,\r\n  VarASCIIList, list of=1 tVarASCII\r\n    VarASCII=1\r\n      VarIdent=GlobHor\r\n      VarType=Irrad\r\n      NoField=2\r\n      FieldMult=1.00000\r\n      UnitId=W_m2\r\n    End of TVarASCII\r\n  End of List VarASCIIList\r\n  LargChamps\r\nEnd of PVObject pvMeteoFormat\r\n";

#[test]
fn should_roundtrip_identically_when_parsing_modern_mef_with_unknown_keys() {
    let mef = parse_mef(FIXTURE_MODERNO_CON_DESCONOCIDO).expect("Parseo exitoso con campo nuevo");
    assert_eq!(mef.comentario(), "Modern Test");
    let recod = encode_mef(&mef).expect("Re-codificación");
    assert_eq!(FIXTURE_MODERNO_CON_DESCONOCIDO, recod.as_slice());
    validar_mef(FIXTURE_MODERNO_CON_DESCONOCIDO).expect("Validación exacta");
}

#[test]
fn should_refuse_to_write_when_legacy_mef_has_no_demonstrated_writer() {
    validar_mef(FIXTURE_LEGACY_MEF).expect_err("legacy no tiene escritor demostrado");
}

#[test]
fn should_decode_latin1_text_when_parsing_legacy_mef() {
    let sample =
        b"pvMeteoFormat;Caf\xE9;\r\nVersion 5.0; Calcul=$00;\r\nGlobHor;Irrad;2;0;1500;1.0;\r\n"
            .to_vec();
    let mef = parse_mef(&sample).expect("parseo de Latin-1");
    assert_eq!(mef.comentario(), "Café");
}

#[test]
fn should_mark_the_read_as_incomplete_when_legacy_mef_has_no_readable_variables() {
    let ws = tempfile::tempdir().expect("Directorio temporal");
    let meteo_dir = ws.path().join("Meteo");
    fs::create_dir_all(&meteo_dir).expect("Crear directorio Meteo");
    fs::write(
        meteo_dir.join("Binario.MEF"),
        b"pvMeteoFormat;Sin Variables;\r\nVersion 5.0; Calcul=$00;\r\n",
    )
    .expect("Escritura del MEF");

    let leido = leer_mef(
        &Paths::para_test(ws.path()),
        &json!({ "archivo": "Binario.MEF" }),
    )
    .expect("Lectura de MEF");

    assert_eq!(leido["total_variables"], 0);
    assert_eq!(leido["completo"], false);
    assert!(leido["aviso"].as_str().is_some_and(|a| !a.is_empty()));
}

#[test]
fn should_reject_path_traversal_when_reading_mef() {
    let ws = tempfile::tempdir().expect("Directorio temporal");
    let paths = Paths::para_test(ws.path());
    leer_mef(&paths, &json!({ "archivo": "../../Meteo.MEF" })).expect_err("traversal");
}
