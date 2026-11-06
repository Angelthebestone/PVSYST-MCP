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
fn should_refuse_writing_files_pvsyst_owns_when_they_are_read_only() {
    let ws = tempdir().expect("ws");
    let paths = dummy_paths(ws.path());

    for (archivo, contenido) in [
        ("Currency.dat", "CURRENCY_RATES_V2;\r\n"),
        (
            "Shadings/Escena.SHD",
            "pvShading;Desc;\r\nVersion 4.0; Calcul=$00;\r\n",
        ),
        (
            "Meteo/Legacy.MEF",
            "pvMeteoFormat;Legacy;\r\nVersion 5.0; Calcul=$00;\r\nGlobHor;Irrad;2;0;1500;1.0;\r\n",
        ),
    ] {
        let err = escribir_archivo(
            &paths,
            &json!({ "archivo": archivo, "contenido": contenido }),
        )
        .expect_err("PVsyst es dueño de este archivo");
        assert!(
            matches!(err, crate::error::Error::Invalid(_)),
            "{archivo}: {err}"
        );
        assert!(
            !ws.path().join(archivo).exists(),
            "{archivo} no debe crearse"
        );
    }
}

#[test]
fn should_refuse_unsafe_file_names_when_writing() {
    let ws = tempdir().expect("ws");
    let paths = dummy_paths(ws.path());

    for archivo in [
        "Meteo/foo.MEF::$DATA",
        "Meteo/foo.MEF.",
        "Meteo/CON.MEF",
        "Meteo/foo\u{0007}.MEF",
    ] {
        let err = escribir_archivo(&paths, &json!({ "archivo": archivo, "contenido": "x" }))
            .expect_err("nombre de archivo inseguro");
        assert!(
            matches!(err, crate::error::Error::Invalid(_)),
            "{archivo}: {err}"
        );
    }
}

#[test]
fn should_write_meteo_bytes_exactly_when_they_reproduce_the_format() {
    let ws = tempdir().expect("ws");
    let paths = dummy_paths(ws.path());
    let bytes = crate::meteo_mef::generar_mef_pvgis_nedo().expect("MEF de PVGIS");

    let args = json!({ "archivo": "Meteo/Mio.MEF", "contenido": base64(&bytes), "base64": true });
    escribir_archivo(&paths, &args).expect("el MEF reproducible se escribe");
    assert_eq!(
        fs::read(ws.path().join("Meteo/Mio.MEF")).expect("leer"),
        bytes
    );

    let bytes_lf: Vec<u8> = bytes.into_iter().filter(|&b| b != b'\r').collect();
    let discrepante = json!({
        "archivo": "Meteo/Discrepante.MEF",
        "contenido": base64(&bytes_lf),
        "base64": true
    });
    let err = escribir_archivo(&paths, &discrepante).expect_err("bytes no reproducibles");
    assert!(matches!(err, crate::error::Error::Invalid(_)), "{err}");
    assert!(!ws.path().join("Meteo/Discrepante.MEF").exists());
}
