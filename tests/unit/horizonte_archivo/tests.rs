use crate::config::Paths;
use crate::error::Error;
use crate::horizonte_archivo::{encode_horizonte, parse_horizonte};
use serde_json::json;
use std::fs;
use std::path::{Path, PathBuf};

fn ruta_muestra_real() -> PathBuf {
    let fixture = Path::new("tests/fixtures/workspace/Shadings/DEMO_Geneve.HOR");
    if fixture.exists() {
        return fixture.to_path_buf();
    }
    crate::config::data_ro().join(r"PVsyst8.1_Data\Shadings\DEMO_Geneve.HOR")
}

#[test]
fn should_preserve_exact_bytes_when_roundtripping_a_real_horizon_sample() {
    let ruta = ruta_muestra_real();
    let bytes = fs::read(&ruta).expect("Lectura de archivo de muestra DEMO_Geneve.HOR");
    let h = parse_horizonte(&bytes).expect("Decodificación de muestra");
    let codificado = encode_horizonte(&h).expect("Codificación de muestra");

    assert_eq!(codificado.len(), bytes.len());
    assert_eq!(codificado, bytes);
}

#[expect(
    clippy::too_many_arguments,
    reason = "deuda medida; pendiente del paso 2 de la limpieza"
)]
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
fn should_write_and_read_a_horizon_when_calling_the_tools_through_the_server() {
    let temp = tempfile::tempdir().expect("tempdir");
    let paths = Paths::para_test(temp.path().to_path_buf());
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

#[test]
fn should_reject_a_horizon_outside_the_workspace_when_the_name_escapes() {
    let ws = tempfile::tempdir().expect("ws");
    let fuera = tempfile::tempdir().expect("fuera");
    let ajeno = fuera.path().join("Ajeno.HOR");
    fs::write(&ajeno, "x").expect("hor");
    let paths = Paths::para_test(ws.path());

    for nombre in [
        ajeno.to_string_lossy().into_owned(),
        "..\\Ajeno.HOR".to_string(),
    ] {
        let res = crate::horizonte_archivo::leer_horizonte(&paths, &json!({ "archivo": nombre }));
        assert!(matches!(res, Err(Error::Invalid(_))), "{nombre}: {res:?}");
    }
}
