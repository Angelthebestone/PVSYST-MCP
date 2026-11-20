use super::*;

fn serie(valores: [f64; 12]) -> SerieMensual {
    SerieMensual::nuevo(&valores).expect("serie válida")
}

fn plana(valor: f64) -> SerieMensual {
    serie([valor; 12])
}

#[test]
fn should_report_twenty_percent_when_measured_is_twenty_percent_higher() {
    let comparacion = comparar(&plana(100.0), &plana(120.0), 10.0);

    assert_eq!(comparacion.meses.len(), 12);
    assert!(comparacion
        .meses
        .iter()
        .all(|fila| fila.desviacion_pct == Some(20.0)));
    assert_eq!(comparacion.anual.mes, 0);
    assert_eq!(comparacion.anual.desviacion_pct, Some(20.0));
    assert_eq!(comparacion.fuera_de_umbral, (1..=12).collect::<Vec<u8>>());
}
