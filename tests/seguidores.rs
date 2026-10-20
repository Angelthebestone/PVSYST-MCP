//! End-to-end tracker conversion over a private copy of the PVsyst demo project.

#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

mod common;

use common::{tool_text, Client};
use serde_json::json;
use std::fs;
use std::path::Path;

const DEMO_PROJECTS: &str = r"C:\Program Files\PVsyst8.1.7\DataRO\PVsyst8.1_Data\Projects";

fn copiar_proyecto(destino: &Path, nombre: &str, variante: &str) {
    let projects = destino.join("Projects");
    fs::create_dir_all(&projects).expect("crear Projects");
    for archivo in [format!("{nombre}.PRJ"), format!("{nombre}.{variante}")] {
        fs::copy(
            Path::new(DEMO_PROJECTS).join(&archivo),
            projects.join(&archivo),
        )
        .expect("copiar demo");
    }
}

#[test]
#[ignore = "needs the PVsyst 8.1.7 demo data: run with --ignored"]
fn should_convert_a_single_orientation_variant_to_a_tracker() {
    let tmp = tempfile::tempdir().expect("tempdir");
    copiar_proyecto(tmp.path(), "_DEMO_Commercial", "VC0");
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
    assert!(!variante.contains("ListeObjets"));

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
