use super::aplicacion::leer_divisas;
use super::codec::parse_currency;
use super::dominio::EntradaDivisa;
use crate::config::Paths;
use serde_json::json;
use std::fs;
use std::path::{Path, PathBuf};

const FIXTURE_CURRENCY: &[u8] =
    b"CURRENCY_RATES_V2;\r\n3; 1;1;\r\nEUR;Euro;    1.0000;\r\nUSD;US Dollar;    1.2500;\r\nJPY;Japanese Yen;  130.0000;";

fn paths_de_test(ws: &Path) -> Paths {
    Paths::para_test(ws)
}

#[test]
fn should_parse_golden_currency_table() {
    let tabla = parse_currency(FIXTURE_CURRENCY).expect("Parseo de tabla golden");
    assert_eq!(tabla.version_formato(), "CURRENCY_RATES_V2");
    assert_eq!(tabla.moneda_base_idx(), 1);
    assert_eq!(tabla.divisas().len(), 3);
    assert_eq!(tabla.divisas()[0].codigo(), "EUR");
    assert_eq!(tabla.divisas()[1].codigo(), "USD");
    assert_eq!(tabla.divisas()[2].codigo(), "JPY");
    assert!((tabla.divisas()[1].tasa_cambio() - 1.25).abs() < 1e-6);
}

#[test]
fn should_reject_invalid_currency_code() {
    let err = EntradaDivisa::nueva("EURO", "Euro".into(), 1.0);
    assert!(err.is_err());
}

#[test]
fn should_reject_negative_exchange_rate() {
    let err = EntradaDivisa::nueva("EUR", "Euro".into(), -1.5);
    assert!(err.is_err());
}

#[test]
fn should_reject_empty_currency_file() {
    let err = parse_currency(b"");
    assert!(err.is_err());
}

#[test]
fn should_reject_currency_table_with_unrecognized_header() {
    let sample = b"CURRENCY_RATES_V2BASURA;\r\n1; 1;1;\r\nEUR;Euro; 1.0;";
    let err = parse_currency(sample);
    assert!(err.is_err());
}

#[test]
fn should_reject_currency_table_with_overflowing_row_count() {
    let sample = b"CURRENCY_RATES_V2;\r\n18446744073709551615; 1;1;\r\nEUR;Euro; 1.0;";
    let err = parse_currency(sample);
    assert!(err.is_err());
}

#[test]
fn should_reject_currency_table_with_row_count_mismatch() {
    let sample = b"CURRENCY_RATES_V2;\r\n5; 1;1;\r\nEUR;Euro; 1.0;\r\nUSD;Dollar; 1.2;";
    let err = parse_currency(sample);
    assert!(err.is_err());
}

#[test]
fn should_reject_currency_row_with_extra_fields() {
    let sample = b"CURRENCY_RATES_V2;\r\n1; 1;1;\r\nEUR;Euro; 1.0;EXTRA;\r\n";
    let err = parse_currency(sample);
    assert!(err.is_err());
}

#[test]
fn should_return_not_found_when_requesting_nonexistent_custom_currency_file() {
    let ws = tempfile::tempdir().expect("Directorio temporal");
    let paths = paths_de_test(ws.path());
    let err = leer_divisas(&paths, &json!({ "archivo": "otro.dat" }));
    assert!(matches!(err, Err(crate::error::Error::NotFound(_))));
}

#[test]
fn should_reject_device_and_colon_names_in_currency_reading() {
    let ws = tempfile::tempdir().expect("Directorio temporal");
    let paths = paths_de_test(ws.path());
    let err_con = leer_divisas(&paths, &json!({ "archivo": "CON.dat" }));
    let err_colon = leer_divisas(&paths, &json!({ "archivo": "C:foo.dat" }));
    assert!(err_con.is_err());
    assert!(err_colon.is_err());
}

#[test]
fn should_list_rates_when_reading_currency_from_workspace() {
    let ws = tempfile::tempdir().expect("Directorio temporal");
    fs::write(ws.path().join("Currency.dat"), FIXTURE_CURRENCY).expect("Escritura");
    let paths = paths_de_test(ws.path());

    let leida = leer_divisas(&paths, &json!({})).expect("Lectura de divisas");
    assert_eq!(leida["total_divisas"], 3);
    assert_eq!(leida["divisas"][1]["codigo"], "USD");
}

#[test]
fn should_reject_path_traversal_when_reading_currency() {
    let ws = tempfile::tempdir().expect("Directorio temporal");
    let paths = paths_de_test(ws.path());
    let err = leer_divisas(&paths, &json!({ "archivo": "../../Currency.dat" }));
    assert!(err.is_err());
}

#[test]
fn should_reject_external_absolute_path_without_permission() {
    let ws = tempfile::tempdir().expect("Directorio temporal");
    let paths = paths_de_test(ws.path());
    let ruta_externa = if cfg!(windows) {
        r"C:\Windows\System32\drivers\etc\hosts"
    } else {
        "/etc/hosts"
    };
    let err = leer_divisas(&paths, &json!({ "archivo": ruta_externa }));
    assert!(err.is_err());
}

#[test]
#[ignore = "necesita la instalación de PVsyst"]
fn should_parse_real_currency_dat_sample() {
    let ruta = PathBuf::from(r"C:\Program Files\PVsyst8.1.7\DataRO\Currency.dat");
    if !ruta.exists() {
        panic!(
            "Falta el archivo requerido de la instalación de PVsyst: {}",
            ruta.display()
        );
    }
    let bytes_orig = fs::read(&ruta).expect("Lectura de Currency.dat real");
    let tabla = parse_currency(&bytes_orig).expect("Parseo de Currency.dat real");
    assert_eq!(tabla.divisas().len(), 33);
    assert_eq!(tabla.moneda_base_idx(), 1);
    assert_eq!(tabla.divisas()[0].codigo(), "EUR");
}
