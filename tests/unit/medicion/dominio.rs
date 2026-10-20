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

#[test]
fn should_leave_the_deviation_empty_when_the_simulated_month_is_zero() {
    let mut valores = [100.0; 12];
    valores[2] = 0.0;
    let comparacion = comparar(&serie(valores), &plana(50.0), 5.0);

    assert_eq!(comparacion.meses[2].desviacion_pct, None);
    assert!(!comparacion.fuera_de_umbral.contains(&3));
}

#[test]
fn should_only_flag_months_strictly_above_the_threshold() {
    let mut medidos = [100.0; 12];
    medidos[0] = 110.0;
    medidos[1] = 111.0;
    let comparacion = comparar(&plana(100.0), &serie(medidos), 10.0);

    assert_eq!(comparacion.fuera_de_umbral, vec![2]);
}

#[test]
fn should_parse_a_header_and_decimal_commas_from_csv() {
    let texto = "mes;energia_kwh\n\
enero;10,5\nfebrero;11,5\nmarzo;12,5\nabril;13,5\nmayo;14,5\njunio;15,5\n\
julio;16,5\nagosto;17,5\nseptiembre;18,5\noctubre;19,5\nnoviembre;20,5\ndiciembre;21,5\n";
    let serie = serie_de_csv(texto).expect("CSV válido");
    assert_eq!(serie.0[0], 10.5);
    assert_eq!(serie.0[11], 21.5);
}

#[test]
fn should_parse_a_csv_without_header() {
    let filas: Vec<String> = (1..=12).map(|mes| format!("{mes};{mes}0")).collect();
    let serie = serie_de_csv(&filas.join("\n")).expect("CSV sin cabecera");
    assert_eq!(serie.0[0], 10.0);
    assert_eq!(serie.0[11], 120.0);
}

#[test]
fn should_reject_csv_with_the_wrong_number_of_rows() {
    let error = serie_de_csv("1;10\n2;20\n").expect_err("pocas filas");
    assert!(matches!(error, Error::Invalid(_)));
    assert!(error.to_string().contains("12"));
}

#[test]
fn should_reject_negative_values() {
    let mut valores = [0.0; 12];
    valores[0] = -1.0;
    let error = SerieMensual::nuevo(&valores).expect_err("negativo");
    assert!(matches!(error, Error::Invalid(_)));
}

#[test]
fn should_reject_a_non_numeric_row_after_the_header() {
    let texto = "mes;energia_kwh\nenero;10\nfebrero;no\n";
    let error = serie_de_csv(texto).expect_err("fila no numérica");
    assert!(matches!(error, Error::Format(_)));
}

#[test]
fn should_reject_a_series_with_nan() {
    let mut valores = [1.0; 12];
    valores[0] = f64::NAN;
    let error = SerieMensual::nuevo(&valores).expect_err("NaN");
    assert!(matches!(error, Error::Invalid(_)));
}
