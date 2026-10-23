use super::escribir_archivo;
use crate::config::Paths;
use crate::workspace::base64::base64;
use serde_json::json;
use std::fs;
use std::path::Path;
use tempfile::tempdir;

fn dummy_paths(ws: &Path) -> Paths {
    Paths::para_test(ws)
}

#[test]
fn should_refuse_writing_currency_dat() {
    let ws = tempdir().expect("ws");
    let paths = dummy_paths(ws.path());
    let args = json!({ "archivo": "Currency.dat", "contenido": "CURRENCY_RATES_V2;\r\n" });

    let err = escribir_archivo(&paths, &args).expect_err("Currency.dat es de solo lectura");
    assert!(matches!(err, crate::error::Error::Invalid(_)), "{err}");
    assert!(err
        .to_string()
        .contains("solo lectura: PVsyst lo lee de su instalación (DataRO)"));
    assert!(!ws.path().join("Currency.dat").exists());
}

#[test]
fn should_refuse_writing_3d_scenes() {
    let ws = tempdir().expect("ws");
    let paths = dummy_paths(ws.path());
    for ext in ["SHD", "BLD", "SHO"] {
        let args = json!({
            "archivo": format!("Shadings/Escena.{ext}"),
            "contenido": "pvShading;Desc;\r\nVersion 4.0; Calcul=$00;\r\n"
        });
        let err = escribir_archivo(&paths, &args).expect_err("escena 3D es solo lectura");
        assert!(matches!(err, crate::error::Error::Invalid(_)), "{err}");
        assert!(err.to_string().contains("escena 3D: solo lectura"));
        assert!(!ws.path().join(format!("Shadings/Escena.{ext}")).exists());
    }
}

#[test]
fn should_refuse_writing_with_colon_or_trailing_dots() {
    let ws = tempdir().expect("ws");
    let paths = dummy_paths(ws.path());
    let args_ads = json!({ "archivo": "Meteo/foo.MEF::$DATA", "contenido": "invalido" });
    let err_ads = escribir_archivo(&paths, &args_ads).expect_err("colon rechazado");
    assert!(matches!(err_ads, crate::error::Error::Invalid(_)));
    assert!(err_ads.to_string().contains(':') || err_ads.to_string().contains("no permitido"));

    let valid_mef = crate::meteo_mef::generar_mef_pvgis_nedo().expect("mef valido");
    let args_dot =
        json!({ "archivo": "Meteo/foo.MEF.", "contenido": base64(&valid_mef), "base64": true });
    let err_dot = escribir_archivo(&paths, &args_dot).expect_err("punto final rechazado");
    assert!(matches!(err_dot, crate::error::Error::Invalid(_)));
    assert!(err_dot.to_string().contains("terminar en punto o espacio"));
}

#[test]
fn should_refuse_writing_windows_device_or_control_chars() {
    let ws = tempdir().expect("ws");
    let paths = dummy_paths(ws.path());
    let err_con = escribir_archivo(
        &paths,
        &json!({ "archivo": "Meteo/CON.MEF", "contenido": "x" }),
    );
    assert!(err_con.is_err());

    let err_ctrl = escribir_archivo(
        &paths,
        &json!({ "archivo": "Meteo/foo\u{0007}.MEF", "contenido": "x" }),
    );
    assert!(err_ctrl.is_err());
}

#[test]
fn should_refuse_writing_content_exceeding_max_bytes_limit() {
    let ws = tempdir().expect("ws");
    let paths = dummy_paths(ws.path());
    // Create a 10 MiB + 1 byte payload without base64
    let oversize = "a".repeat(crate::workspace::MAX_BYTES_MAXIMO + 1);
    let err = escribir_archivo(
        &paths,
        &json!({ "archivo": "Meteo/Grande.MEF", "contenido": oversize }),
    );
    assert!(err.is_err());
    assert!(err
        .unwrap_err()
        .to_string()
        .contains("excede el límite máximo"));
}

#[test]
fn should_refuse_writing_legacy_mef() {
    let ws = tempdir().expect("ws");
    let paths = dummy_paths(ws.path());
    let legacy_content =
        "pvMeteoFormat;Legacy;\r\nVersion 5.0; Calcul=$00;\r\nGlobHor;Irrad;2;0;1500;1.0;\r\n";
    let args = json!({ "archivo": "Meteo/Legacy.MEF", "contenido": legacy_content });

    let err = escribir_archivo(&paths, &args).expect_err("legacy no tiene escritor demostrado");
    assert!(matches!(err, crate::error::Error::Invalid(_)), "{err}");
    assert!(err
        .to_string()
        .contains("no hay escritor legacy demostrado"));
    assert!(!ws.path().join("Meteo/Legacy.MEF").exists());
}

#[test]
fn should_refuse_writing_modern_mef_when_byte_discrepancy_detected() {
    let ws = tempdir().expect("ws");
    let paths = dummy_paths(ws.path());
    let bytes_crlf = crate::meteo_mef::generar_mef_pvgis_nedo().expect("MEF de PVGIS");
    let bytes_lf: Vec<u8> = bytes_crlf.into_iter().filter(|&b| b != b'\r').collect();
    let args = json!({
        "archivo": "Meteo/Discrepante.MEF",
        "contenido": base64(&bytes_lf),
        "base64": true
    });

    let err = escribir_archivo(&paths, &args).expect_err("debe fallar discrepancia exacta");
    assert!(matches!(err, crate::error::Error::Invalid(_)), "{err}");
    assert!(err.to_string().contains("discrepancia"), "{err}");
    assert!(err.to_string().contains("byte offset"), "{err}");
    assert!(!ws.path().join("Meteo/Discrepante.MEF").exists());
}

#[test]
fn should_write_a_meteo_format_when_bytes_reproduce_exactly() {
    let ws = tempdir().expect("ws");
    let paths = dummy_paths(ws.path());
    let bytes = crate::meteo_mef::generar_mef_pvgis_nedo().expect("MEF de PVGIS");
    let args = json!({ "archivo": "Meteo/Mio.MEF", "contenido": base64(&bytes), "base64": true });

    escribir_archivo(&paths, &args).expect("el MEF reproducible se escribe");

    assert_eq!(
        fs::read(ws.path().join("Meteo/Mio.MEF")).expect("leer"),
        bytes
    );
}
