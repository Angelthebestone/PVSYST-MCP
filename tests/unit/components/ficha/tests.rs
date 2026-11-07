//! Tests for datasheet component creation.

use super::*;
use serde_json::json;
use std::fs;

#[test]
#[ignore = "necesita la instalación de PVsyst 8.1 (DataRO): ejecutar con --ignored"]
fn should_apply_fields_to_respective_objects_in_ond() {
    let ws = std::env::temp_dir().join(format!("pvsyst_ficha_{}", std::process::id()));
    let _ = fs::remove_dir_all(&ws);
    let datos = json!({"Manufacturer": "Acme", "Model": "AC-20K", "PNomConv": 20, "VMppMin": 250})
        .as_object()
        .cloned()
        .expect("objeto");
    let r = inversor::ond(&ws, "mcp_tmp_ond", &datos).expect("OND creado");
    let ruta = r["archivo"].as_str().expect("ruta").to_string();
    let texto = String::from_utf8_lossy(&fs::read(&ruta).expect("leer")).into_owned();
    let _ = fs::remove_dir_all(&ws);
    let raiz = crate::format::object::parse_first(&texto).expect("parseable");
    assert_eq!(raiz.find_deep("Model"), Some("AC-20K"));
    assert_eq!(raiz.find_deep("PNomConv"), Some("20"));
    assert_eq!(raiz.find_deep("VMppMin"), Some("250"));
    // The efficiency profile is rescaled by 20/50.
    assert_eq!(raiz.find_deep("Point_2"), Some("600,548"));
}

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
