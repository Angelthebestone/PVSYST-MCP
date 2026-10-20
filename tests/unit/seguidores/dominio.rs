use super::*;

fn entrada(valor: &Value) -> EntradaSeguidor {
    EntradaSeguidor::de_args(valor).expect("argumentos válidos")
}

fn cambio(cambios: &[(String, Option<String>)], clave: &str) -> Option<String> {
    cambios
        .iter()
        .find(|(k, _)| k == clave)
        .and_then(|(_, v)| v.clone())
}

#[test]
fn should_use_demo_defaults_when_no_arguments_are_given() {
    let parametros = ParametrosSeguidor::nuevo(entrada(&json!({}))).expect("defectos");
    let cambios = cambios_orientacion(&parametros, "2");

    assert_eq!(cambio(&cambios, "NoOrient").as_deref(), Some("2"));
    assert_eq!(cambio(&cambios, "AxisAzim").as_deref(), Some("0.0"));
    assert_eq!(cambio(&cambios, "MinPhi").as_deref(), Some("-60.0"));
    assert_eq!(cambio(&cambios, "MaxPhi").as_deref(), Some("60.0"));
    assert_eq!(
        cambio(&cambios, "UsesBacktracking").as_deref(),
        Some("True")
    );
    assert_eq!(cambio(&cambios, "Pitch").as_deref(), Some("8.000"));
    assert_eq!(cambio(&cambios, "CollectorWidth").as_deref(), Some("4.250"));
    assert_eq!(cambio(&cambios, "NSheds").as_deref(), Some("62"));
}

#[test]
fn should_round_angles_and_widths_to_the_pvsyst_precision() {
    let parametros = ParametrosSeguidor::nuevo(entrada(&json!({
        "azimut_eje": -8.04,
        "angulo_max": 55.06,
        "pitch_m": 8.1234,
        "ancho_colector_m": 4.2567,
        "n_filas": 12,
        "backtracking": false
    })))
    .expect("parámetros");
    let cambios = cambios_orientacion(&parametros, "2");

    assert_eq!(cambio(&cambios, "AxisAzim").as_deref(), Some("-8.0"));
    assert_eq!(cambio(&cambios, "MinPhi").as_deref(), Some("-55.1"));
    assert_eq!(cambio(&cambios, "Pitch").as_deref(), Some("8.123"));
    assert_eq!(cambio(&cambios, "CollectorWidth").as_deref(), Some("4.257"));
    assert_eq!(
        cambio(&cambios, "UsesBacktracking").as_deref(),
        Some("False")
    );
    assert_eq!(cambio(&cambios, "NSheds").as_deref(), Some("12"));
}

#[test]
fn should_reject_azimut_outside_its_range() {
    let error = ParametrosSeguidor::nuevo(entrada(&json!({"azimut_eje": 181.0})))
        .expect_err("azimut fuera de rango");
    assert!(matches!(error, Error::Invalid(_)));
    assert!(error.to_string().contains("azimut_eje"));
}

#[test]
fn should_reject_angles_at_or_below_zero_and_above_ninety() {
    for angulo in [0.0, -1.0, 90.1] {
        let error = ParametrosSeguidor::nuevo(entrada(&json!({"angulo_max": angulo})))
            .expect_err("ángulo fuera de rango");
        assert!(matches!(error, Error::Invalid(_)), "ángulo {angulo}");
    }
}

#[test]
fn should_reject_pitch_that_does_not_exceed_the_collector_width() {
    let error = ParametrosSeguidor::nuevo(entrada(&json!({
        "pitch_m": 4.25,
        "ancho_colector_m": 4.25
    })))
    .expect_err("pitch igual al ancho");
    assert!(matches!(error, Error::Invalid(_)));
    assert!(error.to_string().contains("pitch_m"));
}

#[test]
fn should_reject_zero_collector_width_and_zero_rows() {
    let ancho = ParametrosSeguidor::nuevo(entrada(&json!({"ancho_colector_m": 0.0})))
        .expect_err("ancho cero");
    assert!(matches!(ancho, Error::Invalid(_)));

    let filas = ParametrosSeguidor::nuevo(entrada(&json!({"n_filas": 0}))).expect_err("cero filas");
    assert!(matches!(filas, Error::Invalid(_)));
}

#[test]
fn should_reject_non_numeric_or_negative_arguments_at_the_boundary() {
    let texto = EntradaSeguidor::de_args(&json!({"azimut_eje": "oeste"})).expect_err("texto");
    assert!(matches!(texto, Error::Invalid(_)));

    let negativo = EntradaSeguidor::de_args(&json!({"n_filas": -1})).expect_err("negativo");
    assert!(matches!(negativo, Error::Invalid(_)));
}
