//! Stand-alone easy-mode design over a private copy of the demo template.

#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

mod common;

use common::{copiar_proyecto_demo, tool_text, Client};
use serde_json::json;
use std::fs;

#[test]
#[ignore = "needs the PVsyst 8.1.7 demo data: run with --ignored"]
fn should_create_a_stand_alone_variant_with_battery_system_type() {
    let tmp = tempfile::tempdir().expect("tempdir");
    copiar_proyecto_demo(tmp.path(), "_DEMO_StandAlone", "VC3");
    let sites = tmp.path().join("Sites");
    fs::create_dir_all(&sites).expect("Sites");
    fs::write(
        sites.join("Precreated.SIT"),
        "Source=NASA SSE grid cell 7/-74\nLatitude=7.07\nLongitude=-73.84\nAltitude=0\n",
    )
    .expect("sitio reutilizable");
    let mut client = Client::start_env(
        Some(tmp.path().to_str().expect("workspace utf8")),
        &[("PVSYST_MCP_APROXIMACIONES", "1")],
    );
    client.initialize();

    let respuesta = client.call(
        1,
        "disenar_sistema",
        json!({
            "tipo_sistema": "aislado",
            "latitud": 7.07,
            "longitud": -73.84,
            "kwp": 5,
            "simular": false,
            "n_baterias_serie": 2,
            "n_baterias_paralelo": 3
        }),
    );
    let valor = tool_text(&respuesta);
    assert_eq!(valor["tipo_sistema"], "aislado", "{valor}");
    assert_eq!(valor["variante"], "VC3", "{valor}");

    let variante = fs::read_to_string(tmp.path().join("Projects/Sistema_5kWp.VC3"))
        .expect("leer variante aislada");
    assert!(variante.contains("SystemType=Battery"));
    assert!(variante.contains("NBattSerie=2"));
    assert!(variante.contains("NBattParall=3"));
}
