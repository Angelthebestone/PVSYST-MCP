use super::aplicacion::leer_divisas;
use crate::config::Paths;
use serde_json::json;
use std::fs;

const FIXTURE_CURRENCY: &[u8] =
    b"CURRENCY_RATES_V2;\r\n3; 1;1;\r\nEUR;Euro;    1.0000;\r\nUSD;US Dollar;    1.2500;\r\nJPY;Japanese Yen;  130.0000;";

#[test]
fn should_list_rates_when_reading_currency_from_workspace() {
    let ws = tempfile::tempdir().expect("Directorio temporal");
    fs::write(ws.path().join("Currency.dat"), FIXTURE_CURRENCY).expect("Escritura");
    let paths = Paths::para_test(ws.path());

    let leida = leer_divisas(&paths, &json!({})).expect("Lectura de divisas");
    assert_eq!(leida["total_divisas"], 3);
    assert_eq!(leida["divisas"][1]["codigo"], "USD");
}

#[test]
fn should_reject_path_traversal_when_reading_currency() {
    let ws = tempfile::tempdir().expect("Directorio temporal");
    let paths = Paths::para_test(ws.path());
    leer_divisas(&paths, &json!({ "archivo": "../../Currency.dat" })).expect_err("traversal");
}
