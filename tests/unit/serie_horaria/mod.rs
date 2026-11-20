use super::base62::*;
use super::dominio::*;
use crate::error::Error;
use std::fs;

const LINEA_DAY_ABU_DHABI: &str =
    "Day_0101=+0+0+0+0+0+0+0+2E+Tz+10e+1P8+11v+1L8+17U+1Oc+p1+K7+P+0+0+0+0+0+0";
const LINEA_DAY_CHUR_TAMB: &str =
    "Day_0101=-9g-9l-9V-9v-9o-A0-A5-9y-9a-9y-9s-8s-7o-78-6y-6Y-6g-6x-7I-7K-7J-7R-7f-7z";

#[test]
fn should_decode_known_temperatures_when_decoding_a_real_day_line() {
    let (dia, raws) = decodificar_linea_dia(LINEA_DAY_CHUR_TAMB).expect("decode day");
    assert_eq!(dia, "0101");
    assert_eq!(raws.len(), 24);

    let temps: Vec<f64> = raws
        .iter()
        .map(|&r| crudo_a_fisico(r, 100.0, TipoVariable::Temperatura))
        .collect();

    assert!((temps[0] - (-6.00)).abs() < 0.01, "hour 1: {}", temps[0]);
    assert!((temps[1] - (-6.05)).abs() < 0.01, "hour 2: {}", temps[1]);
    assert!((temps[5] - (-6.20)).abs() < 0.01, "hour 6: {}", temps[5]);
    assert!((temps[11] - (-5.50)).abs() < 0.01, "hour 12: {}", temps[11]);
    assert!((temps[13] - (-4.42)).abs() < 0.01, "hour 14: {}", temps[13]);
    assert!((temps[23] - (-4.95)).abs() < 0.01, "hour 24: {}", temps[23]);
}

#[test]
fn should_reencode_byte_identically_when_roundtripping_a_real_day_line() {
    let (dia, raws) = decodificar_linea_dia(LINEA_DAY_ABU_DHABI).expect("decode");
    let reencoded = codificar_linea_dia(&dia, &raws);
    assert_eq!(reencoded.trim(), LINEA_DAY_ABU_DHABI.trim());
}

#[test]
fn should_reject_out_of_range_values_when_validating_a_series() {
    let vars = vec!["GlobHor".to_string(), "T_Amb".to_string()];
    let vals_neg = vec![vec![-5.0; 8760], vec![20.0; 8760]];
    assert!(matches!(
        validar_serie(&vars, &vals_neg),
        Err(crate::error::Error::Invalid(_))
    ));

    let vals_hot = vec![vec![100.0; 8760], vec![120.0; 8760]];
    assert!(matches!(
        validar_serie(&vars, &vals_hot),
        Err(crate::error::Error::Invalid(_))
    ));
}

#[test]
fn should_reject_unknown_variable_when_updating_a_series() {
    let serie = vacia_serie();
    let vars = vec!["ZZ_NoExiste".to_string()];
    let err = super::aplicacion::actualizar_serie(serie, &vars, &[vec![1.0; 8760]])
        .expect_err("unknown variable must be rejected");
    assert!(matches!(err, Error::Invalid(_)), "{err:?}");
    assert!(err.to_string().contains("ZZ_NoExiste"), "{err}");
}

#[test]
fn should_reject_a_series_outside_the_workspace_when_the_name_escapes() {
    let ws = tempfile::tempdir().expect("ws");
    let fuera = tempfile::tempdir().expect("fuera");
    let ajeno = fuera.path().join("Ajeno.MET");
    fs::write(&ajeno, "x").expect("met");
    let paths = crate::config::Paths::para_test(ws.path());

    for nombre in [
        ajeno.to_string_lossy().into_owned(),
        "../Ajeno.MET".to_string(),
    ] {
        let args = serde_json::json!({ "archivo": nombre });
        let res = super::aplicacion::leer_serie(&paths, &args);
        assert!(matches!(res, Err(Error::Invalid(_))), "{nombre}: {res:?}");
    }
}

fn vacia_serie() -> SerieHoraria {
    SerieHoraria {
        top_line: String::new(),
        variables: Vec::new(),
        metadatos: Vec::new(),
        dias: Vec::new(),
        horas: Vec::new(),
        valores_crudos: std::collections::HashMap::new(),
    }
}
