use super::preparacion::con_salidas_por_defecto;
use super::*;

#[test]
fn should_default_outputs_into_results_and_respect_explicit_paths() {
    let dir = tempfile::tempdir().expect("ws");
    let ws = dir.path();
    let v = con_salidas_por_defecto(&json!({}), ws, "Planta", "VC0").expect("por defecto");
    let csv = Path::new(v["output_csv"].as_str().expect("csv"));
    assert!(csv.ends_with("Results/Planta_VC0.csv"));
    assert!(v["report_pdf"]
        .as_str()
        .expect("pdf")
        .ends_with("Planta_VC0.pdf"));

    let explicito = con_salidas_por_defecto(
        &json!({"csv": "x.csv", "sin_pdf": true}),
        ws,
        "Planta",
        "VC0",
    )
    .expect("explícito");
    let csv_explicito = Path::new(explicito["output_csv"].as_str().expect("csv"));
    assert_eq!(csv_explicito, ws.join("x.csv"));
    assert!(explicito.get("report_pdf").is_none());
}
