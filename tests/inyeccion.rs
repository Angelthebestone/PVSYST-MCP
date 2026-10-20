//! End-to-end grid injection limit over a private copy of the demo project.

#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

mod common;

use common::{copiar_proyecto_demo, tool_text, Client};
use serde_json::json;
use std::fs;

#[test]
#[ignore = "needs the PVsyst 8.1.7 demo data: run with --ignored"]
fn should_write_the_injection_limit_into_pv_system() {
    let tmp = tempfile::tempdir().expect("tempdir");
    copiar_proyecto_demo(tmp.path(), "_DEMO_Commercial", "VC0");
    let workspace = tmp.path().to_str().expect("workspace utf8");

    let mut client = Client::start(Some(workspace));
    client.initialize();
    let respuesta = client.call(
        7,
        "proyecto",
        json!({
            "accion": "limite_inyeccion",
            "proyecto": "_DEMO_Commercial",
            "variante": "VC0",
            "potencia_kw": 300
        }),
    );
    let valor = tool_text(&respuesta);
    assert_eq!(valor["parametros"]["potencia_kw"], 300.0, "{valor}");

    let variante = fs::read_to_string(tmp.path().join("Projects/_DEMO_Commercial.VC0"))
        .expect("leer variante");
    assert!(variante.contains("PLimGrid=300.0"));
    assert!(variante.contains("LimGrSeparLoss=False"));
}
