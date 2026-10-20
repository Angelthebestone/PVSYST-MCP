use super::*;

fn parametros(valor: &Value) -> Result<ParametrosInyeccion> {
    ParametrosInyeccion::nuevo(EntradaInyeccion::de_args(valor)?)
}

fn cambio(cambios: &[(String, Option<String>)], clave: &str) -> Option<String> {
    cambios
        .iter()
        .find(|(k, _)| k == clave)
        .and_then(|(_, v)| v.clone())
}

#[test]
fn should_write_the_limit_with_one_decimal_and_no_separate_loss_by_default() {
    let p = parametros(&json!({"potencia_kw": 300})).expect("parámetros");
    let cambios = p.cambios();
    assert_eq!(cambio(&cambios, "PLimGrid").as_deref(), Some("300.0"));
    assert_eq!(cambio(&cambios, "LimGrSeparLoss").as_deref(), Some("False"));
    assert_eq!(cambio(&cambios, "TanPhi"), None);
}

#[test]
fn should_write_separate_loss_and_tan_phi_when_requested() {
    let p = parametros(&json!({
        "potencia_kw": 12.34,
        "perdida_separada": true,
        "tan_phi": 0.4
    }))
    .expect("parámetros");
    let cambios = p.cambios();
    assert_eq!(cambio(&cambios, "PLimGrid").as_deref(), Some("12.3"));
    assert_eq!(cambio(&cambios, "LimGrSeparLoss").as_deref(), Some("True"));
    assert_eq!(cambio(&cambios, "TanPhi").as_deref(), Some("0.400"));
}

#[test]
fn should_reject_a_missing_power_limit() {
    let error = parametros(&json!({})).expect_err("sin potencia");
    assert!(matches!(error, Error::MissingArg { .. }));
    assert!(error.to_string().contains("potencia_kw"));
}

#[test]
fn should_reject_zero_or_negative_power() {
    for potencia in [0.0, -1.0] {
        let error = parametros(&json!({"potencia_kw": potencia})).expect_err("potencia inválida");
        assert!(matches!(error, Error::Invalid(_)), "potencia {potencia}");
    }
}

#[test]
fn should_reject_tan_phi_outside_its_range() {
    for tan_phi in [-1.01, 1.01] {
        let error = parametros(&json!({"potencia_kw": 10, "tan_phi": tan_phi}))
            .expect_err("tan_phi fuera de rango");
        assert!(matches!(error, Error::Invalid(_)), "tan_phi {tan_phi}");
    }
}

#[test]
fn should_reject_a_non_numeric_power_at_the_boundary() {
    let error = EntradaInyeccion::de_args(&json!({"potencia_kw": "mucha"})).expect_err("texto");
    assert!(matches!(error, Error::Invalid(_)));
}
