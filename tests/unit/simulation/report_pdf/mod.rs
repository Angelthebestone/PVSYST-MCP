use super::parse::{parse_energia, parse_numero, parse_pct};
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
fn loss_diagram_gives_temperature_and_nominal_energy() {
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
fn monthly_table_gives_january_grid_energy_and_year_pr() {
    let v = mensual(&fixture()).unwrap();
    assert_eq!(v["meses"][0]["mes"], "January");
    assert_eq!(v["meses"][0]["valores"]["E_Grid"].as_f64(), Some(1344.0));
    assert_eq!(v["anual"]["PR"].as_f64(), Some(0.837));
}

#[test]
fn summary_gives_produced_energy() {
    let v = resumen(&fixture()).unwrap();
    assert_eq!(v["energia_anual_kwh"].as_f64(), Some(14299.0));
    assert_eq!(v["pr_pct"].as_f64(), Some(83.68));
}

#[test]
fn summary_reads_values_when_labels_and_values_are_in_separate_rows() {
    // VC6 prints `Produced Energy`, `Used Energy`, `16539`, `40515` and
    // `Perf. Ratio PR`, `Solar Fraction SF`, `73.31`, `39.09`.
    let v = resumen(&vc6_tokens()).unwrap();

    assert_eq!(v["energia_anual_kwh"].as_f64(), Some(16539.0));
    assert_eq!(v["produccion_especifica_kwh_kwp"].as_f64(), Some(940.0));
    assert_eq!(v["pr_pct"].as_f64(), Some(73.31));
}

#[test]
fn missing_loss_diagram_is_an_error() {
    assert!(perdidas(&["Something else".to_string()]).is_err());
}

#[test]
fn percentage_and_energy_tokens_are_recognised() {
    assert_eq!(parse_pct("+0.9%"), Some(0.9));
    assert_eq!(parse_pct("-10.1%"), Some(-10.1));
    assert_eq!(parse_pct("1.2.3%"), None);
    assert_eq!(parse_energia("1854  kWh/m²"), Some((1854.0, "kWh/m²")));
    assert_eq!(parse_energia("14563 kWh"), Some((14563.0, "kWh")));
    assert_eq!(parse_energia("kWh"), None);
}

#[test]
fn parse_numero_handles_pvsyst_report_number_formats() {
    assert_eq!(parse_numero("23.410.00"), Some(23410.0));
    assert_eq!(parse_numero("1.365.18"), Some(1365.18));
    assert_eq!(parse_numero("1.33"), Some(1.33));
    assert_eq!(parse_numero("9.5"), Some(9.5));
    assert_eq!(parse_numero("30144"), Some(30144.0));
    assert_eq!(parse_numero("-18.5"), Some(-18.5));
    assert_eq!(parse_numero(""), None);
}

#[test]
fn economics_reader_extracts_figures_from_vc6_tokens() {
    let v = economia(&vc6_tokens()).expect("lectura de economía");
    assert_eq!(v["inversion_global"].as_f64(), Some(23410.0));
    assert_eq!(v["moneda"].as_str(), Some("EUR"));
    assert_eq!(v["inversion_especifica"].as_f64(), Some(1.33));
    assert_eq!(v["unidad_especifica"].as_str(), Some("EUR/Wp"));
    assert_eq!(v["anualidades"].as_f64(), Some(680.28));
    assert_eq!(v["anualidades_unidad"].as_str(), Some("EUR/yr"));
    assert_eq!(v["costes_operacion"].as_f64(), Some(1365.18));
    assert_eq!(v["costes_operacion_unidad"].as_str(), Some("EUR/yr"));
    assert_eq!(v["payback_anios"].as_f64(), Some(9.5));
    assert_eq!(v["lcoe"].as_f64(), Some(0.17));
    assert_eq!(v["lcoe_unidad"].as_str(), Some("EUR/kWh"));
}

#[test]
fn carbon_reader_extracts_totals_and_params_from_vc6_tokens() {
    let v = carbono(&vc6_tokens()).expect("lectura de carbono");
    assert_eq!(v["emisiones_evitadas_total"].as_f64(), Some(13.6));
    assert_eq!(v["emisiones_evitadas_unidad"].as_str(), Some("tCO2"));
    assert_eq!(v["emisiones_ciclo_vida_total"].as_f64(), Some(30.23));
    assert_eq!(v["emisiones_ciclo_vida_unidad"].as_str(), Some("tCO2"));
    assert_eq!(v["balance_neto_total"].as_f64(), Some(-18.5));
    assert_eq!(v["balance_neto_unidad"].as_str(), Some("tCO2"));
    assert_eq!(v["produccion_sistema"].as_f64(), Some(16.74));
    assert_eq!(v["produccion_sistema_unidad"].as_str(), Some("MWh/yr"));
    assert_eq!(v["factor_lce_red"].as_f64(), Some(27.0));
    assert_eq!(v["factor_lce_red_unidad"].as_str(), Some("gCO2/kWh"));
    assert_eq!(v["fuente"].as_str(), Some("IEA List"));
    assert_eq!(v["pais"].as_str(), Some("Switzerland"));
    assert_eq!(v["vida_util_anios"].as_f64(), Some(30.0));
    assert_eq!(v["degradacion_anual_pct"].as_f64(), Some(1.0));
}

#[test]
fn carbon_reader_extracts_lce_details_from_vc6_tokens() {
    let v = carbono(&vc6_tokens()).expect("lectura de carbono");
    let detalles = v["detalles_lce"].as_array().expect("array de detalles");
    assert_eq!(detalles.len(), 3);
    assert_eq!(detalles[0]["concepto"].as_str(), Some("Modules"));
    assert_eq!(detalles[0]["subtotal_kgco2"].as_f64(), Some(30144.0));
    assert_eq!(detalles[1]["concepto"].as_str(), Some("Supports"));
    assert_eq!(detalles[1]["subtotal_kgco2"].as_f64(), Some(72.0));
    assert_eq!(detalles[2]["concepto"].as_str(), Some("Inverters"));
    assert_eq!(detalles[2]["subtotal_kgco2"].as_f64(), Some(17.8));
}

#[test]
fn missing_economics_block_returns_helpful_error() {
    let err = economia(&fixture()).unwrap_err();
    let msg = err.to_string();
    assert!(
        msg.contains("proyecto accion:economia"),
        "mensaje esperado con instrucciones, recibido: {msg}"
    );
}

#[test]
fn missing_carbon_page_returns_helpful_error() {
    let err = carbono(&fixture()).unwrap_err();
    let msg = err.to_string();
    assert!(
        msg.contains("PrintInReport"),
        "mensaje esperado con instrucciones, recibido: {msg}"
    );
}
