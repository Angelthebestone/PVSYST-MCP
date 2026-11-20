use super::*;

fn entrada(valor: &Value) -> EntradaSeguidor {
    EntradaSeguidor::de_args(valor).expect("argumentos válidos")
}

#[test]
fn should_reject_azimut_outside_its_range() {
    ParametrosSeguidor::nuevo(entrada(&json!({"azimut_eje": 181.0})))
        .expect_err("azimut fuera de rango");
}

#[test]
fn should_reject_pitch_that_does_not_exceed_the_collector_width() {
    ParametrosSeguidor::nuevo(entrada(&json!({
        "pitch_m": 4.25,
        "ancho_colector_m": 4.25
    })))
    .expect_err("pitch igual al ancho");
}
