use crate::config::Paths;
use crate::error::Error;
use crate::horizonte_archivo::{encode_horizonte, parse_horizonte, Horizonte, PuntoHorizonte};
use serde_json::json;
use std::fs;
use std::path::{Path, PathBuf};

fn ruta_muestra_real() -> PathBuf {
    let fixture = Path::new("tests/fixtures/workspace/Shadings/DEMO_Geneve.HOR");
    if fixture.exists() {
        return fixture.to_path_buf();
    }
    PathBuf::from(r"C:\Program Files\PVsyst8.1.7\DataRO\PVsyst8.1_Data\Shadings\DEMO_Geneve.HOR")
}

#[test]
fn should_decode_real_sample_and_match_points() {
    let ruta = ruta_muestra_real();
    let bytes = fs::read(&ruta).expect("Lectura de archivo de muestra DEMO_Geneve.HOR");
    let h = parse_horizonte(&bytes).expect("Decodificación exitosa de DEMO_Geneve.HOR");

    assert_eq!(h.titulo, "Horizon line at Genève");
    assert_eq!(h.version, "3.0");
    assert_eq!(h.calcul, "$0001");
    assert_eq!(h.puntos.len(), 12);

    // Assert first and last points: -120° and +120° at 0° height
    let primero = &h.puntos[0];
    assert!((primero.azimut - (-120.0)).abs() < 1e-3);
    assert!((primero.altura - 0.0).abs() < 1e-3);

    let ultimo = &h.puntos[11];
    assert!((ultimo.azimut - 120.0).abs() < 1e-3);
    assert!((ultimo.altura - 0.0).abs() < 1e-3);

    // Assert representative interior points
    let p1 = &h.puntos[1];
    assert!((p1.azimut - (-92.0)).abs() < 1e-3);
    assert!((p1.altura - 6.1).abs() < 1e-3);

    let p6 = &h.puntos[6];
    assert!((p6.azimut - (-12.3)).abs() < 1e-3);
    assert!((p6.altura - 10.4).abs() < 1e-3);
}

#[test]
fn should_preserve_exact_bytes_on_real_sample_roundtrip() {
    let ruta = ruta_muestra_real();
    let bytes = fs::read(&ruta).expect("Lectura de archivo de muestra DEMO_Geneve.HOR");
    let h = parse_horizonte(&bytes).expect("Decodificación de muestra");
    let codificado = encode_horizonte(&h).expect("Codificación de muestra");

    assert_eq!(codificado.len(), bytes.len());
    assert_eq!(codificado, bytes);
}

#[test]
fn should_roundtrip_synthetic_horizon() {
    let puntos = vec![
        PuntoHorizonte::de_grados(-180.0, 0.0),
        PuntoHorizonte::de_grados(-90.0, 8.5),
        PuntoHorizonte::de_grados(0.0, 15.0),
        PuntoHorizonte::de_grados(90.0, 8.5),
        PuntoHorizonte::de_grados(180.0, 0.0),
    ];
    let sintetico = Horizonte {
        titulo: "Synthetic Test Horizon".to_string(),
        version: "3.0".to_string(),
        calcul: "$0001".to_string(),
        puntos: puntos.clone(),
    };

    let bytes = encode_horizonte(&sintetico).expect("Codificación horizonte sintético");
    let dec = parse_horizonte(&bytes).expect("Decodificación horizonte sintético");

    assert_eq!(dec.titulo, sintetico.titulo);
    assert_eq!(dec.version, sintetico.version);
    assert_eq!(dec.calcul, sintetico.calcul);
    assert_eq!(dec.puntos.len(), sintetico.puntos.len());

    for (p_orig, p_dec) in puntos.iter().zip(dec.puntos.iter()) {
        assert!((p_orig.azimut - p_dec.azimut).abs() < 1e-3);
        assert!((p_orig.altura - p_dec.altura).abs() < 1e-3);
    }

    // Re-encoding decoded synthetic must be byte-for-byte identical
    let re_codificado = encode_horizonte(&dec).expect("Re-codificación");
    assert_eq!(re_codificado, bytes);
}

#[test]
fn should_reject_truncated_input() {
    assert!(parse_horizonte(&[]).is_err());
    assert!(parse_horizonte(&[0xd7, 0x04]).is_err());

    let ruta = ruta_muestra_real();
    let bytes = fs::read(&ruta).expect("Lectura de muestra");
    // Truncate halfway through header
    assert!(parse_horizonte(&bytes[..40]).is_err());
    // Truncate in binary body
    assert!(parse_horizonte(&bytes[..100]).is_err());
}

#[test]
fn should_reject_invalid_magic() {
    let mut bytes = fs::read(ruta_muestra_real()).expect("Lectura de muestra");
    bytes[0] = 0x00;
    bytes[1] = 0x00;
    match parse_horizonte(&bytes) {
        Err(Error::Format(msg)) => assert!(msg.contains("firma")),
        other => panic!("Esperaba Error::Format por firma incorrecta, obtenido {other:?}"),
    }
}

#[test]
fn should_reject_out_of_range_azimuth() {
    let puntos_bajos = vec![
        PuntoHorizonte::de_grados(-190.0, 0.0),
        PuntoHorizonte::de_grados(0.0, 10.0),
    ];
    let h1 = Horizonte {
        titulo: "Bad".into(),
        version: "3.0".into(),
        calcul: "$0001".into(),
        puntos: puntos_bajos,
    };
    assert!(matches!(encode_horizonte(&h1), Err(Error::Invalid(_))));

    let puntos_altos = vec![
        PuntoHorizonte::de_grados(0.0, 0.0),
        PuntoHorizonte::de_grados(190.0, 10.0),
    ];
    let h2 = Horizonte {
        titulo: "Bad".into(),
        version: "3.0".into(),
        calcul: "$0001".into(),
        puntos: puntos_altos,
    };
    assert!(matches!(encode_horizonte(&h2), Err(Error::Invalid(_))));
}

#[test]
fn should_reject_out_of_range_height() {
    let puntos_negativos = vec![
        PuntoHorizonte::de_grados(-10.0, -1.0),
        PuntoHorizonte::de_grados(10.0, 10.0),
    ];
    let h1 = Horizonte {
        titulo: "Bad".into(),
        version: "3.0".into(),
        calcul: "$0001".into(),
        puntos: puntos_negativos,
    };
    assert!(matches!(encode_horizonte(&h1), Err(Error::Invalid(_))));

    let puntos_excesivos = vec![
        PuntoHorizonte::de_grados(-10.0, 0.0),
        PuntoHorizonte::de_grados(10.0, 95.0),
    ];
    let h2 = Horizonte {
        titulo: "Bad".into(),
        version: "3.0".into(),
        calcul: "$0001".into(),
        puntos: puntos_excesivos,
    };
    assert!(matches!(encode_horizonte(&h2), Err(Error::Invalid(_))));
}

#[test]
fn should_reject_non_increasing_azimuth() {
    let puntos = vec![
        PuntoHorizonte::de_grados(10.0, 5.0),
        PuntoHorizonte::de_grados(10.0, 8.0),
    ];
    let h = Horizonte {
        titulo: "Bad".into(),
        version: "3.0".into(),
        calcul: "$0001".into(),
        puntos,
    };
    assert!(matches!(encode_horizonte(&h), Err(Error::Invalid(_))));
}

#[test]
fn should_reject_too_few_points() {
    let puntos = vec![PuntoHorizonte::de_grados(0.0, 5.0)];
    let h = Horizonte {
        titulo: "Bad".into(),
        version: "3.0".into(),
        calcul: "$0001".into(),
        puntos,
    };
    assert!(matches!(encode_horizonte(&h), Err(Error::Invalid(_))));
}

#[test]
fn should_read_and_write_via_application() {
    let temp = tempfile::tempdir().expect("tempdir");
    let mut paths = Paths::resolve();
    paths.workspace = temp.path().to_path_buf();

    let args_escribir = json!({
        "archivo": "TestHorizonte.HOR",
        "titulo": "Horizonte de prueba",
        "puntos": [
            { "azimut": -100.0, "altura": 2.0 },
            { "azimut": 0.0, "altura": 8.0 },
            { "azimut": 100.0, "altura": 1.0 }
        ]
    });

    let res_escritura = crate::horizonte_archivo::escribir_horizonte(&paths, &args_escribir)
        .expect("Escritura de horizonte");
    assert_eq!(res_escritura["puntos"], 3);

    let args_leer = json!({
        "archivo": "TestHorizonte.HOR"
    });
    let res_lectura =
        crate::horizonte_archivo::leer_horizonte(&paths, &args_leer).expect("Lectura de horizonte");
    assert_eq!(res_lectura["titulo"], "Horizonte de prueba");
    assert_eq!(res_lectura["puntos"].as_array().expect("array").len(), 3);
}

fn invocar_tool_json(
    server: &crate::Server,
    id: u64,
    name: &str,
    args: &serde_json::Value,
) -> serde_json::Value {
    let req = json!({
        "jsonrpc": "2.0",
        "id": id,
        "method": "tools/call",
        "params": { "name": name, "arguments": args }
    });
    let out = server.handle_line(&req.to_string()).expect("respuesta");
    let resp: serde_json::Value = serde_json::from_str(&out).expect("json");
    assert!(
        !resp["result"]["isError"].as_bool().unwrap_or(false),
        "error en {name}: {resp}"
    );
    resp
}

#[test]
fn should_execute_tools_e2e_via_server() {
    let temp = tempfile::tempdir().expect("tempdir");
    let mut paths = Paths::resolve();
    paths.workspace = temp.path().to_path_buf();
    let server = crate::Server::new(paths);

    invocar_tool_json(
        &server,
        1,
        "sitio",
        &json!({
            "accion": "escribir_horizonte",
            "archivo": "E2E_Horizon.HOR",
            "titulo": "Horizonte E2E",
            "puntos": [
                { "azimut": -90.0, "altura": 5.0 },
                { "azimut": 0.0, "altura": 10.0 },
                { "azimut": 90.0, "altura": 5.0 }
            ]
        }),
    );

    let resp_leer = invocar_tool_json(&server, 2, "leer", &json!({ "archivo": "E2E_Horizon.HOR" }));
    let text = resp_leer["result"]["content"][0]["text"]
        .as_str()
        .expect("text");
    let parsed: serde_json::Value = serde_json::from_str(text).expect("parsed");
    assert_eq!(parsed["titulo"], "Horizonte E2E");
    assert_eq!(parsed["puntos"].as_array().expect("puntos").len(), 3);
}
