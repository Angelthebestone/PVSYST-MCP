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
#[test]
fn should_pass_synthetic_weather_to_the_simulation_step_as_swg() {
    // Given: the arguments the easy-mode pipeline builds for its simulation step
    let args = json!({ "proyecto": "Planta", "variante": "VC0", "site": "Sitio.SIT",
                       "synthetic_weather": true });

    // When
    let mut v = Vec::new();
    push_options(&args, &mut v);

    // Then: PVsystCLI receives the flag that generates the synthetic weather
    assert!(v.contains(&"-swg".to_string()), "args={v:?}");
    let mut sin = Vec::new();
    push_options(&json!({ "synthetic_weather": false }), &mut sin);
    assert!(!sin.contains(&"-swg".to_string()), "args={sin:?}");
}
