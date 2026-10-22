//! End-to-end PVsyst economic evaluation over private copies of the demos.

#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

mod cliente_mcp;

use cliente_mcp::{copiar_proyecto_demo, tool_text, Client};
use serde_json::json;
use std::fs;

fn proyecto_demo(nombre: &str, variante: &str) -> tempfile::TempDir {
    let tmp = tempfile::tempdir().expect("tempdir");
    copiar_proyecto_demo(tmp.path(), nombre, variante);
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
fn should_update_every_discount_rate_and_the_other_economic_fields() {
    let tmp = proyecto_demo("_DEMO_Commercial", "VC9");
    let mut client = cliente(&tmp);

    let respuesta = client.call(
        7,
        "proyecto",
        json!({
            "accion": "economia",
            "proyecto": "_DEMO_Commercial",
            "variante": "VC9",
            "moneda": "COP",
            "vida_anios": 25,
            "tasa_descuento": 0.03,
            "inflacion": 0.02,
            "tarifa_venta_kwh": 0.15
        }),
    );
    let valor = tool_text(&respuesta);
    assert_eq!(valor["parametros"]["moneda"], "COP", "{valor}");

    let variante = fs::read_to_string(tmp.path().join("Projects/_DEMO_Commercial.VC9"))
        .expect("leer variante");
    assert_eq!(variante.matches("DiscountRate=0.0300").count(), 2);
    assert!(variante.contains("Currency=COP"));
    assert!(variante.contains("ProjectLife=25"));
    assert!(variante.contains("InflationRate=0.0200"));
    assert!(variante.contains("SummerPeakSaleTariff=0.15000"));
}

#[test]
#[ignore = "needs the PVsyst 8.1.7 demo data: run with --ignored"]
fn should_report_not_found_when_the_variant_has_no_economic_evaluation() {
    let tmp = proyecto_demo("_DEMO_Commercial", "VC0");
    let mut client = cliente(&tmp);

    let respuesta = client.call(
        8,
        "proyecto",
        json!({
            "accion": "economia",
            "proyecto": "_DEMO_Commercial",
            "variante": "VC0",
            "tasa_descuento": 0.03
        }),
    );
    assert_eq!(respuesta["result"]["isError"], true);
    let mensaje = respuesta["result"]["content"][0]["text"]
        .as_str()
        .expect("mensaje de error");
    assert!(mensaje.contains("evaluación económica"), "{mensaje}");
}

#[test]
#[ignore = "needs the PVsyst 8.1.7 demo data: run with --ignored"]
fn should_reject_a_feed_in_tariff_when_the_tariff_is_variable() {
    let tmp = proyecto_demo("_DEMO_Residential", "VC6");
    let mut client = cliente(&tmp);

    let respuesta = client.call(
        9,
        "proyecto",
        json!({
            "accion": "economia",
            "proyecto": "_DEMO_Residential",
            "variante": "VC6",
            "tarifa_venta_kwh": 0.15
        }),
    );
    assert_eq!(respuesta["result"]["isError"], true);
    let mensaje = respuesta["result"]["content"][0]["text"]
        .as_str()
        .expect("mensaje de error");
    assert!(mensaje.contains("Tarifa variable"), "{mensaje}");
}
