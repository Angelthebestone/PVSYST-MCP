//! End-to-end battery storage over a private copy of the demo project.

#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

mod common;

use common::{copiar_proyecto_demo, tool_text, Client};
use serde_json::json;
use std::fs;

fn proyecto_demo() -> tempfile::TempDir {
    let tmp = tempfile::tempdir().expect("tempdir");
    copiar_proyecto_demo(tmp.path(), "_DEMO_Commercial", "VC0");
    tmp
}

fn cliente(tmp: &tempfile::TempDir) -> Client {
    let workspace = tmp.path().to_str().expect("workspace utf8");
    let mut client = Client::start(Some(workspace));
    client.initialize();
    client
}

#[test]
#[ignore = "needs the PVsyst 8.1.7 demo data: run with --ignored"]
fn should_reject_peak_shaving_without_an_injection_limit() {
    let tmp = proyecto_demo();
    let mut client = cliente(&tmp);

    let respuesta = client.call(
        7,
        "proyecto",
        json!({
            "accion": "almacenamiento",
            "proyecto": "_DEMO_Commercial",
            "variante": "VC0",
            "estrategia": "recorte_picos"
        }),
    );
    assert_eq!(respuesta["result"]["isError"], true);
    let mensaje = respuesta["result"]["content"][0]["text"]
        .as_str()
        .expect("mensaje de error");
    assert!(mensaje.contains("PLimGrid"), "{mensaje}");
}

#[test]
#[ignore = "needs the PVsyst 8.1.7 demo data: run with --ignored"]
fn should_insert_storage_and_apply_peak_shaving_after_the_limit() {
    let tmp = proyecto_demo();
    let mut client = cliente(&tmp);

    let limite = client.call(
        8,
        "proyecto",
        json!({
            "accion": "limite_inyeccion",
            "proyecto": "_DEMO_Commercial",
            "variante": "VC0",
            "potencia_kw": 300
        }),
    );
    assert_eq!(limite["result"]["isError"], false);

    let respuesta = client.call(
        9,
        "proyecto",
        json!({
            "accion": "almacenamiento",
            "proyecto": "_DEMO_Commercial",
            "variante": "VC0",
            "estrategia": "recorte_picos"
        }),
    );
    let valor = tool_text(&respuesta);
    assert_eq!(
        valor["parametros"]["estrategia"], "recorte_picos",
        "{valor}"
    );
    let insertados = valor["insertados"].as_array().expect("insertados");
    assert!(
        insertados.iter().any(|objeto| objeto["tipo"] == "pvLoad"),
        "{valor}"
    );
    assert!(
        insertados
            .iter()
            .any(|objeto| objeto["tipo"] == "pvGridStorage"),
        "{valor}"
    );

    let variante = fs::read_to_string(tmp.path().join("Projects/_DEMO_Commercial.VC0"))
        .expect("leer variante");
    assert!(variante.contains("PVObject_GridStorage=pvGridStorage"));
    assert!(variante.contains("Strategy=PeakShaving"));
    assert!(variante.contains("BatteryFile=Generic_LI_Module_180Ah.BTR"));
    assert!(variante.contains("NBattSerie=10"));
    assert!(variante.contains("NBattParall=20"));
    assert!(variante.contains("BattSoCInit=0.600"));
    assert!(variante.contains("PMaxCharge=380.000"));
    assert!(variante.contains("PMaxDisch=170.000"));
}
