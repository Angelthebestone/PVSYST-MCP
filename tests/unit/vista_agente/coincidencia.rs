use crate::vista_agente::dominio::CampoEsquema;
use crate::vista_agente::esquemas::entidad;
use crate::vista_agente::plan::coincide;

fn campo(entidad_clave: &str, clave: &str) -> &'static CampoEsquema {
    entidad(entidad_clave)
        .expect("entity")
        .campo(clave)
        .expect("field")
}

#[test]
fn should_accept_a_decimal_number_when_pvsyst_shows_fewer_decimals() {
    let kwp = campo("variante.system", "potencia_planeada_kwp");

    assert!(coincide(kwp, "12.345", "12.3"));
    assert!(coincide(kwp, "12.35", "12.4"));
    assert!(coincide(kwp, "410.5", "410"));
}

#[test]
fn should_reject_a_large_number_when_the_difference_is_more_than_the_display_rounding() {
    let kwp = campo("variante.system", "potencia_planeada_kwp");

    assert!(!coincide(kwp, "100", "100.9"));
    assert!(!coincide(kwp, "100", "101"));
    assert!(!coincide(kwp, "12.345", "12.30"));
}

#[test]
fn should_accept_the_same_number_when_pvsyst_writes_trailing_zeros() {
    let kwp = campo("variante.system", "potencia_planeada_kwp");

    assert!(coincide(kwp, "200", "200.0"));
    assert!(coincide(kwp, "10,5", "10.50"));
}

#[test]
fn should_require_an_exact_count_when_the_field_is_an_integer() {
    let cadenas = campo("variante.system", "n_cadenas");

    assert!(coincide(cadenas, "4", "4"));
    assert!(coincide(cadenas, "4", "4.0"));
    assert!(!coincide(cadenas, "4", "3"));
    assert!(!coincide(cadenas, "4", "4.4"));
}

#[test]
fn should_reject_a_number_when_pvsyst_shows_something_that_is_not_one() {
    let kwp = campo("variante.system", "potencia_planeada_kwp");

    assert!(!coincide(kwp, "10", ""));
    assert!(!coincide(kwp, "10", "n/a"));
}
