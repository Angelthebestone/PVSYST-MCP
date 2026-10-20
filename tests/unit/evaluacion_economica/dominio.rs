use super::*;

fn parametros(valor: &Value) -> Result<ParametrosEconomicos> {
    ParametrosEconomicos::nuevo(EntradaEconomicos::de_args(valor)?)
}

fn valor(cambios: &[(String, Option<String>)], clave: &str) -> Option<String> {
    cambios
        .iter()
        .find(|(k, _)| k == clave)
        .and_then(|(_, v)| v.clone())
}

#[test]
fn should_map_every_economic_field_to_the_section() {
    let p = parametros(&json!({
        "moneda": "USD",
        "vida_anios": 20,
        "tasa_descuento": 0.03,
        "inflacion": 0.015,
        "tarifa_venta_kwh": 0.15
    }))
    .expect("parámetros");
    let cambios = p.cambios();

    assert_eq!(valor(&cambios, "Currency").as_deref(), Some("USD"));
    assert_eq!(valor(&cambios, "ProjectLife").as_deref(), Some("20"));
    assert_eq!(valor(&cambios, "DiscountRate").as_deref(), Some("0.0300"));
    assert_eq!(valor(&cambios, "InflationRate").as_deref(), Some("0.0150"));
    assert_eq!(
        valor(&cambios, "SummerPeakSaleTariff").as_deref(),
        Some("0.15000")
    );
}

#[test]
fn should_accept_a_partial_request() {
    let p = parametros(&json!({"tasa_descuento": 0.05})).expect("parámetros");
    assert_eq!(p.cambios().len(), 1);
    assert_eq!(
        valor(&p.cambios(), "DiscountRate").as_deref(),
        Some("0.0500")
    );
}

#[test]
fn should_reject_a_request_without_any_economic_field() {
    let error = parametros(&json!({})).expect_err("sin campos");
    assert!(matches!(error, Error::MissingArg { .. }));
    assert!(error.to_string().contains("moneda"));
}

#[test]
fn should_reject_currencies_that_are_not_three_uppercase_letters() {
    for moneda in ["US", "usd", "US1", "USDX"] {
        let error = parametros(&json!({"moneda": moneda})).expect_err("moneda inválida");
        assert!(matches!(error, Error::Invalid(_)), "moneda {moneda}");
    }
}

#[test]
fn should_reject_life_outside_one_to_one_hundred() {
    for vida in [0, 101] {
        let error = parametros(&json!({"vida_anios": vida})).expect_err("vida inválida");
        assert!(matches!(error, Error::Invalid(_)), "vida {vida}");
    }
}

#[test]
fn should_reject_rates_and_tariffs_out_of_range() {
    for entrada in [
        json!({"tasa_descuento": -0.1}),
        json!({"tasa_descuento": 1.1}),
        json!({"inflacion": -0.1}),
        json!({"inflacion": 1.1}),
        json!({"tarifa_venta_kwh": -1}),
        json!({"tarifa_venta_kwh": 101}),
    ] {
        let error = parametros(&entrada).expect_err("fuera de rango");
        assert!(matches!(error, Error::Invalid(_)), "{entrada}");
    }
}
