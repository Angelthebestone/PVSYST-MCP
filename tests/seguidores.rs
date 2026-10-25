//! End-to-end tracker conversion over a private copy of the PVsyst demo project.
#![expect(
    clippy::indexing_slicing,
    clippy::too_many_arguments,
    reason = "deuda medida; pendiente del paso 2 de la limpieza"
)]
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

mod cliente_mcp;

use cliente_mcp::{copiar_proyecto_demo, tool_text, Client};
use serde_json::json;
use std::fs;

#[test]
#[ignore = "needs the PVsyst 8.1.7 demo data: run with --ignored"]
fn should_convert_a_single_orientation_variant_to_a_tracker() {
    let tmp = tempfile::tempdir().expect("tempdir");
    copiar_proyecto_demo(tmp.path(), "_DEMO_Commercial", "VC0");
    let workspace = tmp.path().to_str().expect("workspace utf8");

    let mut client = Client::start(Some(workspace));
    client.initialize();
    let respuesta = client.call(
        7,
        "proyecto",
        json!({
            "accion": "seguidor",
            "proyecto": "_DEMO_Commercial",
            "variante": "VC0",
            "angulo_max": 55
        }),
    );
    let valor = tool_text(&respuesta);
    assert_eq!(valor["tipo"], "UnlimTracking", "{valor}");

    let variante = fs::read_to_string(tmp.path().join("Projects/_DEMO_Commercial.VC0"))
        .expect("leer variante");
    assert!(variante.contains("FieldType=UnlimTracking"));
    assert!(variante.contains("MinPhi=-55.0"));
    assert!(variante.contains("NoOrient=1"));
    assert!(variante.contains("Flags=$20080000"));
    assert!(!variante.contains("Flags=$000A0000"));

    let doctor = client.call(
        8,
        "proyecto",
        json!({
            "accion": "doctor",
            "proyecto": "_DEMO_Commercial",
            "variante": "VC0"
        }),
    );
    let informe = tool_text(&doctor);
    assert_eq!(informe["ok"], true, "doctor: {informe}");
}

fn cliente_demo(proyecto: &str, variante: &str) -> (tempfile::TempDir, Client) {
    let tmp = tempfile::tempdir().expect("tempdir");
    copiar_proyecto_demo(tmp.path(), proyecto, variante);
    let mut client = Client::start(Some(tmp.path().to_str().expect("workspace utf8")));
    client.initialize();
    (tmp, client)
}

fn error_de_seguidor(client: &mut Client, id: u64, proyecto: &str, variante: &str) -> String {
    let respuesta = client.call(
        id,
        "proyecto",
        json!({
            "accion": "seguidor",
            "proyecto": proyecto,
            "variante": variante
        }),
    );
    assert_eq!(respuesta["result"]["isError"], true, "{respuesta}");
    respuesta["result"]["content"][0]["text"]
        .as_str()
        .expect("mensaje de error")
        .to_string()
}

#[test]
#[ignore = "needs the PVsyst 8.1.7 demo data: run with --ignored"]
fn should_reject_a_3d_shading_scene() {
    let (_tmp, mut client) = cliente_demo("_DEMO_Commercial", "VC5");
    let mensaje = error_de_seguidor(&mut client, 11, "_DEMO_Commercial", "VC5");
    assert!(mensaje.contains("escena 3D"), "{mensaje}");
}

#[test]
#[ignore = "needs the PVsyst 8.1.7 demo data: run with --ignored"]
fn should_reject_a_bifacial_shed_model() {
    let (_tmp, mut client) = cliente_demo("_DEMO_Utility", "VC2");
    let mensaje = error_de_seguidor(&mut client, 12, "_DEMO_Utility", "VC2");
    assert!(mensaje.contains("bifacial"), "{mensaje}");
}

#[test]
#[ignore = "needs the PVsyst 8.1.7 demo data: run with --ignored"]
fn should_reject_a_variant_with_more_than_one_orientation() {
    let tmp = tempfile::tempdir().expect("tempdir");
    copiar_proyecto_demo(tmp.path(), "_DEMO_Commercial", "VC0");
    let ruta = tmp.path().join("Projects/_DEMO_Commercial.VC0");
    let texto = fs::read_to_string(&ruta).expect("leer variante");
    let texto = texto.replace("Orientations, list of=1", "Orientations, list of=2");
    fs::write(&ruta, texto).expect("escribir variante");

    let mut client = Client::start(Some(tmp.path().to_str().expect("workspace utf8")));
    client.initialize();
    let mensaje = error_de_seguidor(&mut client, 13, "_DEMO_Commercial", "VC0");
    assert!(mensaje.contains("una orientación"), "{mensaje}");
}
