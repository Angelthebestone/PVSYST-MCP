use super::aplicacion::{generar_mef_pvgis_nedo, leer_mef, validar_mef};
use super::codec::{encode_mef, parse_mef};
use super::dominio::{DatosVariableMef, MeteoMef, VariableMef};
use crate::config::Paths;
use serde_json::json;
use std::fs;
use std::path::Path;

const FIXTURE_LEGACY_MEF: &[u8] = b"pvMeteoFormat;Sample Legacy Format;\r\nVersion 5.67; Calcul=$00003043;\r\nDelim=;\r\nGlobHor;Irrad;2;0;1500;1.0;\r\nT_Amb;Temper;3;-40;60;1.0;\r\nWindVel;WindVel;4;0;50;1.0;\r\n";

const FIXTURE_MODERNO_CON_DESCONOCIDO: &[u8] = b"\xef\xbb\xbfPVObject_=pvMeteoFormat\r\n  Comment=Modern Test\r\n  Version=8.1.0\r\n  DateFmtType=MM_DD_hh_mm\r\n  Separ=$003B\r\n  CampoDesconocido=ValorArbitrario\r\n  MeasureStep=60\r\n  AlbedoSolMF=0.20\r\n  NoFieldDate=1,2,0,\r\n  VarASCIIList, list of=1 tVarASCII\r\n    VarASCII=1\r\n      VarIdent=GlobHor\r\n      VarType=Irrad\r\n      NoField=2\r\n      FieldMult=1.00000\r\n      UnitId=W_m2\r\n    End of TVarASCII\r\n  End of List VarASCIIList\r\n  LargChamps\r\nEnd of PVObject pvMeteoFormat\r\n";

fn paths_de_test(ws: &Path) -> Paths {
    Paths::para_test(ws)
}

#[test]
fn should_parse_golden_legacy_mef_and_clean_prefixes() {
    let mef = parse_mef(FIXTURE_LEGACY_MEF).expect("Parseo de MEF legacy");
    assert_eq!(mef.comentario(), "Sample Legacy Format");
    assert_eq!(mef.version(), "5.67");
    if let MeteoMef::Legacy(l) = &mef {
        assert_eq!(l.calcul(), "$00003043");
    } else {
        panic!("Se esperaba variante Legacy");
    }

    let vars = mef.variables();
    assert_eq!(vars.len(), 3);
    assert_eq!(vars[0].ident(), "GlobHor");
    assert_eq!(vars[0].unidad(), "W_m2");
    assert_eq!(vars[1].ident(), "T_Amb");
    assert_eq!(vars[1].unidad(), "DegC");
    assert_eq!(vars[2].ident(), "WindVel");
    assert_eq!(vars[2].unidad(), "m_s");
}

#[test]
fn should_refuse_to_validate_or_write_legacy_mef() {
    let err = validar_mef(FIXTURE_LEGACY_MEF).expect_err("legacy no tiene escritor demostrado");
    assert!(err
        .to_string()
        .contains("no hay escritor legacy demostrado"));
}

#[test]
fn should_tolerate_unknown_keys_in_modern_mef_and_roundtrip_identically() {
    let mef = parse_mef(FIXTURE_MODERNO_CON_DESCONOCIDO).expect("Parseo exitoso con campo nuevo");
    assert_eq!(mef.comentario(), "Modern Test");
    let recod = encode_mef(&mef).expect("Re-codificación");
    assert_eq!(FIXTURE_MODERNO_CON_DESCONOCIDO, recod.as_slice());
    validar_mef(FIXTURE_MODERNO_CON_DESCONOCIDO).expect("Validación exacta");
}

#[test]
fn should_reject_modern_mef_missing_closing_tag_or_required_fields() {
    let sin_cierre = b"PVObject_=pvMeteoFormat\r\nComment=Test\r\nVersion=8.1\r\n";
    assert!(parse_mef(sin_cierre).is_err());
}

#[test]
fn should_reject_modern_mef_with_count_mismatch_in_var_list() {
    let mismatch = b"PVObject_=pvMeteoFormat\r\n  Comment=T\r\n  Version=8.1\r\n  DateFmtType=MM_DD\r\n  Separ=$003B\r\n  MeasureStep=60\r\n  AlbedoSolMF=0.2\r\n  NoFieldDate=1,2,0,\r\n  VarASCIIList, list of=2 tVarASCII\r\n    VarASCII=1\r\n      VarIdent=GlobHor\r\n      VarType=Irrad\r\n      NoField=3\r\n      FieldMult=1.0\r\n      UnitId=W_m2\r\n    End of TVarASCII\r\n  End of List VarASCIIList\r\nEnd of PVObject pvMeteoFormat\r\n";
    assert!(parse_mef(mismatch).is_err());
}

#[test]
fn should_reject_invalid_albedo_values() {
    for alb in ["NaN", "inf", "-0.1", "1.5"] {
        let contenido = format!("PVObject_=pvMeteoFormat\r\n  Comment=T\r\n  Version=8.1\r\n  DateFmtType=MM_DD\r\n  Separ=$003B\r\n  MeasureStep=60\r\n  AlbedoSolMF={alb}\r\n  NoFieldDate=1,2,0,\r\n  VarASCIIList, list of=1 tVarASCII\r\n    VarASCII=1\r\n      VarIdent=GlobHor\r\n      VarType=Irrad\r\n      NoField=3\r\n      FieldMult=1.0\r\n      UnitId=W_m2\r\n    End of TVarASCII\r\n  End of List VarASCIIList\r\nEnd of PVObject pvMeteoFormat\r\n");
        assert!(
            parse_mef(contenido.as_bytes()).is_err(),
            "debió fallar para albedo {alb}"
        );
    }
}

#[test]
fn should_reject_duplicate_positive_columns_in_no_field_date() {
    let dup = b"PVObject_=pvMeteoFormat\r\n  Comment=T\r\n  Version=8.1\r\n  DateFmtType=MM_DD\r\n  Separ=$003B\r\n  MeasureStep=60\r\n  AlbedoSolMF=0.2\r\n  NoFieldDate=2,2,3,0,\r\n  VarASCIIList, list of=1 tVarASCII\r\n    VarASCII=1\r\n      VarIdent=GlobHor\r\n      VarType=Irrad\r\n      NoField=4\r\n      FieldMult=1.0\r\n      UnitId=W_m2\r\n    End of TVarASCII\r\n  End of List VarASCIIList\r\nEnd of PVObject pvMeteoFormat\r\n";
    assert!(parse_mef(dup).is_err());
}

#[test]
fn should_parse_legacy_mef_without_text_variables_for_binary_delphi_formats() {
    let sin_vars = b"pvMeteoFormat;Sin Variables;\r\nVersion 5.0; Calcul=$00;\r\n";
    let mef = parse_mef(sin_vars).expect("válido para formatos con variables binarias");
    assert_eq!(mef.variables().len(), 0);
}

#[test]
fn should_parse_legacy_mef_with_latin1_text() {
    let sample =
        b"pvMeteoFormat;Caf\xE9;\r\nVersion 5.0; Calcul=$00;\r\nGlobHor;Irrad;2;0;1500;1.0;\r\n"
            .to_vec();
    let mef = parse_mef(&sample).expect("parseo de Latin-1");
    assert_eq!(mef.comentario(), "Café");
}
#[test]
fn should_reject_invalid_variable_column() {
    let err = VariableMef::nueva(DatosVariableMef {
        ident: "GlobHor".into(),
        tipo: "Irrad".into(),
        columna: 0,
        multiplicador: 1.0,
        unidad: "W_m2".into(),
    });
    assert!(err.is_err());
}

#[test]
fn should_generate_valid_pvgis_nedo_mef() {
    let bytes = generar_mef_pvgis_nedo().expect("Generación exitosa");
    let mef = parse_mef(&bytes).expect("Parseo exitoso del MEF generado");
    assert_eq!(mef.comentario(), "ANE-NEDO known format");
    assert_eq!(mef.variables().len(), 4);
    assert_eq!(mef.variables()[0].ident(), "GlobHor");
    assert_eq!(mef.variables()[0].columna(), 6);
    assert_eq!(mef.variables()[1].ident(), "DiffHor");
    assert_eq!(mef.variables()[1].columna(), 7);
    assert_eq!(mef.variables()[2].ident(), "T_Amb");
    assert_eq!(mef.variables()[2].columna(), 8);
    assert_eq!(mef.variables()[3].ident(), "WindVel");
    assert_eq!(mef.variables()[3].columna(), 9);
}

#[test]
fn should_list_columns_when_reading_mef_from_workspace() {
    let ws = tempfile::tempdir().expect("Directorio temporal");
    let meteo_dir = ws.path().join("Meteo");
    fs::create_dir_all(&meteo_dir).expect("Crear directorio Meteo");
    fs::write(
        meteo_dir.join("Mio.MEF"),
        generar_mef_pvgis_nedo().expect("Generación exitosa"),
    )
    .expect("Escritura del MEF");
    let paths = paths_de_test(ws.path());

    let leido = leer_mef(&paths, &json!({ "archivo": "Mio.MEF" })).expect("Lectura de MEF");
    assert_eq!(leido["comentario"], "ANE-NEDO known format");
    assert_eq!(leido["total_variables"], 4);
}

#[test]
fn should_reject_path_traversal_when_reading_mef() {
    let ws = tempfile::tempdir().expect("Directorio temporal");
    let paths = paths_de_test(ws.path());
    let err = leer_mef(&paths, &json!({ "archivo": "../../Meteo.MEF" }));
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
    let err = leer_mef(&paths, &json!({ "archivo": ruta_externa }));
    assert!(err.is_err());
}

#[test]
#[ignore = "necesita la instalación de PVsyst"]
fn should_roundtrip_exact_bytes_on_real_modern_mef_samples() {
    let muestras = [
        r"C:\Program Files\PVsyst8.1.7\DataRO\PVsyst8.1_Data\Meteo\PVsyst_ANE-NEDO.MEF",
        r"C:\Program Files\PVsyst8.1.7\DataRO\PVsyst8.1_Data\CLI\Ressources\Examples\Sources\WeatherData\Hawaii_2020_hourly.MEF",
    ];

    for ruta_str in muestras {
        let ruta = Path::new(ruta_str);
        if !ruta.exists() {
            panic!(
                "Falta el archivo requerido de la instalación de PVsyst: {}",
                ruta.display()
            );
        }
        let bytes_orig = fs::read(ruta).expect("Lectura de muestra real");
        let mef = parse_mef(&bytes_orig).expect("Parseo exitoso de muestra MEF");
        let bytes_recod = encode_mef(&mef).expect("Re-codificación de muestra MEF");
        assert_eq!(
            bytes_orig,
            bytes_recod,
            "Roundtrip debe ser 100% idéntico byte a byte para {}",
            ruta.file_name().unwrap().to_string_lossy()
        );
    }
}

#[test]
#[ignore = "necesita la instalación de PVsyst"]
fn should_parse_and_reject_writing_on_real_legacy_mef_samples() {
    let muestras_legacy = [
        r"C:\Program Files\PVsyst8.1.7\DataRO\PVsyst8.1_Data\Meteo\PVsyst_Standard_GHI_DNI.MEF",
        r"C:\Program Files\PVsyst8.1.7\DataRO\PVsyst8.1_Data\Meteo\PVsyst_Standard_GHI_DHI.MEF",
        r"C:\Program Files\PVsyst8.1.7\DataRO\PVsyst8.1_Data\Meteo\SIG_Geneva.MEF",
    ];
    for ruta_str in muestras_legacy {
        let ruta = Path::new(ruta_str);
        if !ruta.exists() {
            panic!(
                "Falta el archivo requerido de la instalación de PVsyst: {}",
                ruta.display()
            );
        }
        let bytes_orig = fs::read(ruta).expect("Lectura de muestra real");
        let mef = parse_mef(&bytes_orig).expect("Parseo exitoso de muestra MEF legacy");
        assert!(matches!(mef, MeteoMef::Legacy(_)));
        let err = validar_mef(&bytes_orig).expect_err("legacy no puede validarse para escribir");
        assert!(err
            .to_string()
            .contains("no hay escritor legacy demostrado"));
    }
}
