use super::*;
use crate::error::Error;
use std::fs;
use std::path::PathBuf;

const LINEA_DAY_ABU_DHABI: &str =
    "Day_0101=+0+0+0+0+0+0+0+2E+Tz+10e+1P8+11v+1L8+17U+1Oc+p1+K7+P+0+0+0+0+0+0";
const LINEA_DAY_CHUR_TAMB: &str =
    "Day_0101=-9g-9l-9V-9v-9o-A0-A5-9y-9a-9y-9s-8s-7o-78-6y-6Y-6g-6x-7I-7K-7J-7R-7f-7z";

#[test]
fn should_decode_real_file_day_against_known_values() {
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
fn should_reencode_real_day_byte_identically() {
    let (dia, raws) = decodificar_linea_dia(LINEA_DAY_ABU_DHABI).expect("decode");
    let reencoded = codificar_linea_dia(&dia, &raws);
    assert_eq!(reencoded.trim(), LINEA_DAY_ABU_DHABI.trim());
}

#[test]
fn should_reject_malformed_base62_alphabet() {
    assert!(decodificar_b62("").is_err());
    assert!(decodificar_b62("12@4").is_err());
    assert!(decodificar_b62("abc!").is_err());
    assert!(decodificar_linea_dia("Day_0101=+0+12?+4").is_err());
}

#[test]
fn should_reject_invalid_day_line_length() {
    assert!(decodificar_linea_dia("Day_0101=+0+1").is_err());
    assert!(decodificar_linea_dia("NotADay=+0").is_err());
    assert!(decodificar_linea_dia("Day_abcd=+0").is_err());
}

#[test]
fn should_reject_invalid_series_length() {
    let vars = vec!["GlobHor".to_string()];
    let vals = vec![vec![100.0; 500]];
    let err = validar_serie(&vars, &vals).expect_err("should reject 500 hours");
    assert!(matches!(err, Error::Invalid(_)));
}

#[test]
fn should_reject_out_of_range_physical_values() {
    let vars = vec!["GlobHor".to_string(), "T_Amb".to_string()];
    let vals_neg = vec![vec![-5.0; 8760], vec![20.0; 8760]];
    assert!(validar_serie(&vars, &vals_neg).is_err());

    let vals_hot = vec![vec![100.0; 8760], vec![120.0; 8760]];
    assert!(validar_serie(&vars, &vals_hot).is_err());
}

#[test]
fn should_roundtrip_synthetic_series() {
    let vars = vec!["GlobHor".to_string(), "T_Amb".to_string()];
    let gh_vals: Vec<f64> = (0..8760)
        .map(|h| {
            if (6..18).contains(&(h % 24)) {
                500.0
            } else {
                0.0
            }
        })
        .collect();
    let ta_vals: Vec<f64> = (0..8760).map(|h| 15.0 + ((h % 24) as f64) * 0.5).collect();
    validar_serie(&vars, &[gh_vals.clone(), ta_vals.clone()]).expect("valid series");

    let wr_gh = 2777.778;
    let wr_ta = 100.0;
    let gh_raws: Vec<i64> = gh_vals
        .iter()
        .map(|&v| fisico_a_crudo(v, wr_gh, TipoVariable::Irradiacion))
        .collect();
    let ta_raws: Vec<i64> = ta_vals
        .iter()
        .map(|&v| fisico_a_crudo(v, wr_ta, TipoVariable::Temperatura))
        .collect();

    for h in 0..24 {
        let gh_back = crudo_a_fisico(gh_raws[h], wr_gh, TipoVariable::Irradiacion);
        let ta_back = crudo_a_fisico(ta_raws[h], wr_ta, TipoVariable::Temperatura);
        assert!((gh_back - gh_vals[h]).abs() <= 0.1, "gh hour {h}");
        assert!((ta_back - ta_vals[h]).abs() <= 0.01, "ta hour {h}");
    }
}

#[test]
fn should_decode_real_met_matching_annual_monthly_totals() {
    let ro_path = PathBuf::from(
        r"C:\Program Files\PVsyst8.1.7\DataRO\PVsyst8.1_Data\Meteo\Abu Dhabi DEMO Project_MN82_SYN.MET",
    );
    if !ro_path.exists() {
        return;
    }
    let content = fs::read_to_string(&ro_path).expect("read MET");
    let serie = decodificar(&content).expect("decode MET");

    assert_eq!(serie.horas.len(), 8760);
    assert!(serie.variables.contains(&"GlobHor".to_string()));
    assert!(serie.variables.contains(&"T_Amb".to_string()));

    let total_gh_kwh: f64 = serie
        .horas
        .iter()
        .map(|h| h.valores.get("GlobHor").copied().unwrap_or(0.0))
        .sum::<f64>()
        / 1000.0;

    let mean_ta: f64 = serie
        .horas
        .iter()
        .map(|h| h.valores.get("T_Amb").copied().unwrap_or(0.0))
        .sum::<f64>()
        / 8760.0;

    assert!(
        (total_gh_kwh - 2141.6).abs() < 1.0,
        "total gh: {total_gh_kwh}"
    );
    assert!((mean_ta - 28.50).abs() < 0.1, "mean ta: {mean_ta}");
}

#[test]
fn should_reencode_entire_met_body_byte_identically() {
    let ro_path = PathBuf::from(
        r"C:\Program Files\PVsyst8.1.7\DataRO\PVsyst8.1_Data\Meteo\Abu Dhabi DEMO Project_MN82_SYN.MET",
    );
    if !ro_path.exists() {
        return;
    }
    let content = fs::read_to_string(&ro_path).expect("read MET");
    let pos_start = content.find("  VarMeteo, list of=").unwrap();
    let end_marker = "  End of the List VarMeteo\r\n";
    let pos_end = content.find(end_marker).unwrap() + end_marker.len();
    let body = &content[pos_start..pos_end];

    let serie = decodificar(body).expect("decode body");
    let reencoded = codificar(&serie).expect("encode body");
    assert_eq!(reencoded, body);
}
