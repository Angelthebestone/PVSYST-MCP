use super::*;

fn parametros(valor: &Value) -> Result<ParametrosEconomicos> {
    ParametrosEconomicos::nuevo(EntradaEconomicos::de_args(valor)?)
}

#[test]
fn should_reject_currencies_when_they_are_not_three_uppercase_letters() {
    for moneda in ["US", "usd", "US1", "USDX"] {
        parametros(&json!({"moneda": moneda})).expect_err("moneda inválida");
    }
}

#[test]
fn should_reject_rates_and_tariffs_when_they_are_out_of_range() {
    for entrada in [
        json!({"tasa_descuento": -0.1}),
        json!({"tasa_descuento": 1.1}),
        json!({"inflacion": -0.1}),
        json!({"inflacion": 1.1}),
        json!({"tarifa_venta_kwh": -1}),
        json!({"tarifa_venta_kwh": 1e9}),
    ] {
        parametros(&entrada).expect_err("fuera de rango");
    }
}
