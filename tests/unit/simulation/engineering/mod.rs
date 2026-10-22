use super::caso::pais_simulado;
use super::comparacion::excluida;
use super::incertidumbre::{coef_variacion, incertidumbre};

#[test]
fn coef_variacion_of_two_values() {
    assert_eq!(coef_variacion(&[100.0, 100.0]), Some(0.0));
    let cv = coef_variacion(&[90.0, 110.0]).unwrap();
    assert!((cv - 0.1414).abs() < 1e-3);
    assert_eq!(coef_variacion(&[100.0]), None);
}

#[test]
fn p90_and_p99_use_combined_sigma() {
    let v = incertidumbre(1000.0, 5.0, 3.0, "defecto");
    assert!((v["p90_kwh"].as_f64().unwrap() - 925.3).abs() < 0.5);
    assert!((v["p99_kwh"].as_f64().unwrap() - 864.4).abs() < 0.5);
}

#[test]
fn simulated_country_comes_from_results_csv() {
    let csv = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/tests/fixtures/results/MCP_E2E.csv"
    ));
    assert_eq!(pais_simulado(csv).as_deref(), Some("Colombia"));
    assert_eq!(pais_simulado("Project;X;\n"), None);
}

#[test]
fn excluded_parameter_prefixes_are_skipped() {
    assert!(excluida("VarSimul_TempLss"));
    assert!(!excluida("FieldTilt"));
}
