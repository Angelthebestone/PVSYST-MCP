use super::*;

fn parametros(valor: &Value) -> Result<ParametrosInyeccion> {
    ParametrosInyeccion::nuevo(EntradaInyeccion::de_args(valor)?)
}

#[test]
fn should_reject_power_when_it_is_zero_or_negative() {
    for potencia in [0.0, -1.0] {
        parametros(&json!({"potencia_kw": potencia})).expect_err("potencia inválida");
    }
}

#[test]
fn should_reject_tan_phi_when_it_is_outside_its_range() {
    for tan_phi in [-1.01, 1.01] {
        parametros(&json!({"potencia_kw": 10, "tan_phi": tan_phi}))
            .expect_err("tan_phi fuera de rango");
    }
}
