use super::*;

fn fixture() -> Vec<String> {
    let path = concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/tests/fixtures/results/MCP_E2E.pdf"
    );
    tokens(&std::fs::read(path).unwrap())
}

fn vc6_tokens() -> Vec<String> {
    let raw = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/tests/fixtures/results/vc6_tokens.json"
    ));
    serde_json::from_str(raw).expect("vc6_tokens.json válido")
}

#[test]
fn should_read_temperature_loss_and_nominal_energy_from_the_loss_diagram() {
    let v = perdidas(&fixture()).unwrap();
    let nodos = v["nodos"].as_array().unwrap();
    let temp = nodos
        .iter()
        .find(|n| n["concepto"] == "PV loss due to temperature")
        .unwrap();
    assert_eq!(temp["pct"].as_f64(), Some(-10.1));
    let nominal = nodos
        .iter()
        .find(|n| n["concepto"] == "Array nominal energy (at STC effic.)")
        .unwrap();
    assert_eq!(nominal["valor"].as_f64(), Some(16693.0));
    assert!(v["resumen"]
        .as_str()
        .unwrap()
        .starts_with("−10.1 % temperatura"));
}

#[test]
fn should_read_summary_figures_when_labels_and_values_are_in_separate_rows() {
    // VC6 prints `Produced Energy`, `Used Energy`, `16539`, `40515` and
    // `Perf. Ratio PR`, `Solar Fraction SF`, `73.31`, `39.09`.
    let v = resumen(&vc6_tokens()).unwrap();

    assert_eq!(v["energia_anual_kwh"].as_f64(), Some(16539.0));
    assert_eq!(v["produccion_especifica_kwh_kwp"].as_f64(), Some(940.0));
    assert_eq!(v["pr_pct"].as_f64(), Some(73.31));
}
