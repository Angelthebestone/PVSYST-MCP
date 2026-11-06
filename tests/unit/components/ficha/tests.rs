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
