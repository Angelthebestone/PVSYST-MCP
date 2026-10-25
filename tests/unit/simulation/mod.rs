use super::preparacion::{con_salidas_por_defecto, push_options};
use super::*;

#[test]
fn typed_options_are_emitted() {
    let mut v = Vec::new();
    let args = json!({
        "input_rvt_file": "in.RVT",
        "cmd_options_file": "opts.cof",
        "recalcular_sombreado": false,
        "paginas_reporte": ["cover", "summary"],
        "idioma_reporte": "es",
    });
    push_options(&args, &mut v);
    assert!(v.contains(&"-irf:in.RVT".to_string()));
    assert!(v.contains(&"-cof:opts.cof".to_string()));
    assert!(v.contains(&"-rst:0".to_string()));
    assert!(v.contains(&"-rp:cover,summary".to_string()));
    assert!(v.contains(&"-rl:es".to_string()));
}

#[test]
fn timeout_default_for_simulation_is_900() {
    assert_eq!(
        cli::timeout_de(&json!({}), cli::TIMEOUT_SIMULACION_S),
        std::time::Duration::from_secs(900)
    );
}

#[test]
fn default_outputs_point_into_results_and_respect_overrides() {
    let ws = Path::new("C:/ws");
    let v = con_salidas_por_defecto(&json!({}), ws, "Planta", "VC0");
    let csv = Path::new(v["output_csv"].as_str().unwrap());
    assert!(csv.ends_with("Results/Planta_VC0.csv"));
    assert!(v["report_pdf"]
        .as_str()
        .unwrap()
        .ends_with("Planta_VC0.pdf"));

    let explicito = con_salidas_por_defecto(
        &json!({"csv": "x.csv", "sin_pdf": true}),
        ws,
        "Planta",
        "VC0",
    );
    assert!(explicito.get("output_csv").is_none());
    assert!(explicito.get("report_pdf").is_none());
}

#[test]
fn modo_gui_enum_matches_de_bool() {
    assert!(ModoGui::de_bool(true).debe_abrir());
    assert!(!ModoGui::de_bool(false).debe_abrir());
}
