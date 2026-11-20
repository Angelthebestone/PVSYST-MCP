//! Tests for datasheet component creation.

use super::*;
use serde_json::json;

/// A complete inverter datasheet, minus what each case changes.
fn ficha_ond() -> serde_json::Map<String, Value> {
    json!({
        "Model": "AC-20K", "Manufacturer": "Acme", "PNomConv": 20, "VOutConv": 400,
        "VMppMin": 250, "VMPPMax": 800, "VAbsMax": 1000, "PSeuil": 1.0,
        "EfficMax": 98.0, "EfficEuro": 97.0, "MonoTri": "Tri"
    })
    .as_object()
    .cloned()
    .expect("objeto")
}

#[test]
fn should_require_the_grid_voltage_and_write_it_when_creating_an_inverter_from_a_datasheet() {
    let mut sin_red = ficha_ond();
    sin_red.remove("VOutConv");
    let error = datos::validar(&datos::INVERSOR, &sin_red).expect_err("sin VOutConv");
    assert!(error.to_string().contains("VOutConv"), "error={error}");

    let valores = datos::valores_de(&datos::INVERSOR, &ficha_ond(), "AC-20K");
    assert_eq!(valores["voltaje_red_v"], json!(400));

    let mut en_vatios = ficha_ond();
    en_vatios.insert("PNomConv".into(), json!(50_000));
    let error = datos::validar(&datos::INVERSOR, &en_vatios).expect_err("kW fuera de rango");
    assert!(error.to_string().contains("PNomConv"), "error={error}");
}

#[test]
fn should_refuse_text_in_a_numeric_datasheet_field_when_creating_from_a_ficha() {
    let mut datos = ficha_ond();
    datos.insert("PNomConv".into(), json!("abc"));

    let error = datos::validar(&datos::INVERSOR, &datos).expect_err("PNomConv no numérico");
    assert!(error.to_string().contains("PNomConv"), "error={error}");
}
