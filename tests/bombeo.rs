//! End-to-end pumping configuration over a private copy of the demo project.

#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

mod cliente_mcp;

use cliente_mcp::{copiar_proyecto_demo, tool_text, Client};
use serde_json::json;
use std::fs;

#[test]
#[ignore = "needs the PVsyst 8.1.7 demo data: run with --ignored"]
fn should_write_flow_depth_and_pump_to_system_and_water_use() {
    let tmp = tempfile::tempdir().expect("tempdir");
    copiar_proyecto_demo(tmp.path(), "_DEMO_Pumping", "VC0");
    let workspace = tmp.path().to_str().expect("workspace utf8");

    let mut client = Client::start(Some(workspace));
    client.initialize();
    let respuesta = client.call(
        7,
        "proyecto",
        json!({
            "accion": "bombeo",
            "proyecto": "_DEMO_Pumping",
            "variante": "VC0",
            "caudal_m3_dia": 25,
            "profundidad_estatica_m": 80,
            "bomba": "Generic_6kW_H60_160_FR10.PMP"
        }),
    );
    let valor = tool_text(&respuesta);
    assert_eq!(valor["parametros"]["caudal_m3_dia"], 25.0, "{valor}");

    let variante =
        fs::read_to_string(tmp.path().join("Projects/_DEMO_Pumping.VC0")).expect("leer variante");
    assert!(variante.contains("FlowModul=Yearly"));
    assert!(variante.contains("FlowYear=25.000"));
    assert_eq!(variante.matches("StaticDepth=-80.0").count(), 2);
    assert!(variante.contains("PumpFile=Generic_6kW_H60_160_FR10.PMP"));
}
