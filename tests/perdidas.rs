//! End-to-end variant losses over private copies of the demo projects.

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
fn should_apply_losses_to_main_and_both_subarrays() {
    let tmp = proyecto_demo("_DEMO_Tracking", "VC5");
    let mut client = cliente(&tmp);

    let respuesta = client.call(
        7,
        "proyecto",
        json!({
            "accion": "perdidas",
            "proyecto": "_DEMO_Tracking",
            "variante": "VC5",
            "suciedad_pct": 25,
            "uc_termico": 29,
            "lid_pct": 1.5,
            "mismatch_pct": 2.25
        }),
    );
    let valor = tool_text(&respuesta);
    assert_eq!(valor["n_subarrays"], 2, "{valor}");

    let variante =
        fs::read_to_string(tmp.path().join("Projects/_DEMO_Tracking.VC5")).expect("leer variante");
    assert!(variante.contains("SoilingLoss=0.250"));
    assert!(variante.contains("UFactor=29.0"));
    assert_eq!(variante.matches("LIDLoss=1.50").count(), 2);
    assert_eq!(variante.matches("PMismMPP=2.25").count(), 2);
}

#[test]
#[ignore = "needs the PVsyst 8.1.7 demo data: run with --ignored"]
fn should_reject_soiling_when_the_variant_uses_the_new_model() {
    let tmp = proyecto_demo("_DEMO_Tracking", "VC5");
    let ruta = tmp.path().join("Projects/_DEMO_Tracking.VC5");
    let texto = fs::read_to_string(&ruta).expect("leer variante");
    let con_modelo = texto.replace(
        "  End of PVObject pvMainArray",
        "    Soiling=Soiling\n  End of PVObject pvMainArray",
    );
    fs::write(&ruta, con_modelo).expect("escribir variante");

    let mut client = cliente(&tmp);
    let respuesta = client.call(
        8,
        "proyecto",
        json!({
            "accion": "perdidas",
            "proyecto": "_DEMO_Tracking",
            "variante": "VC5",
            "suciedad_pct": 10
        }),
    );
    assert_eq!(respuesta["result"]["isError"], true);
    let mensaje = respuesta["result"]["content"][0]["text"]
        .as_str()
        .expect("mensaje de error");
    assert!(mensaje.contains("Formato nuevo de suciedad"), "{mensaje}");
}
