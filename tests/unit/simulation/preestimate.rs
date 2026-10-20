use super::*;

fn colombia() -> Sitio {
    Sitio {
        latitud: 7.07,
        longitud: -73.84,
        zona_horaria: -5.0,
    }
}

fn clima_constante() -> Clima {
    Clima::nuevo(vec![5.0; 12], vec![2.0; 12], vec![25.0; 12]).expect("12 meses")
}

#[test]
fn la_rejilla_tiene_169_orientaciones_e_incluye_el_plano() {
    let grid = rejilla();
    assert_eq!(grid.len(), 169);
    assert!(grid.contains(&Orientacion {
        inclinacion: 0.0,
        azimut: 0.0
    }));
}

#[test]
fn el_ranking_va_de_mayor_a_menor_rendimiento_especifico() {
    let top = ranking(&colombia(), &clima_constante(), 9.0, TOP_PREESTIMACION);
    assert_eq!(top.len(), TOP_PREESTIMACION);
    assert!(top
        .windows(2)
        .all(|w| w[0].1.especifica >= w[1].1.especifica));
}

#[test]
fn numero_de_orientaciones_acepta_solo_enteros_en_el_rango() {
    assert_eq!(numero_de_orientaciones(&json!({})).ok(), Some(3));
    assert_eq!(numero_de_orientaciones(&json!({"n": 5})).ok(), Some(5));
    assert!(numero_de_orientaciones(&json!({"n": 0})).is_err());
    assert!(numero_de_orientaciones(&json!({"n": 2.5})).is_err());
    assert!(numero_de_orientaciones(&json!({"n": 170})).is_err());
}

#[test]
fn kwp_debe_ser_positivo_y_admite_texto_numerico() {
    assert!(kwp_de(&json!({"kwp": 0})).is_err());
    assert!(kwp_de(&json!({"kwp": -3})).is_err());
    assert_eq!(kwp_de(&json!({"kwp": "9"})).ok(), Some(9.0));
}
