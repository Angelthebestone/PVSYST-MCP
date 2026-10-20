//! Tests for datasheet component creation.

use super::inversor::puntos_escalados;
use super::*;
use serde_json::json;
use std::fs;

#[test]
fn should_scale_points_both_axes_and_skip_unrelated_keys() {
    let texto = "NPtsEff=2\n  Point_1=248,0\n  Point_2=1500,1370\n  NbInputs=4\n";
    let cambios = puntos_escalados(texto, 2.0);
    assert_eq!(cambios.len(), 2);
    assert_eq!(cambios[0].0, "Point_1");
    assert_eq!(cambios[0].1.as_deref(), Some("496,0"));
    assert_eq!(cambios[1].1.as_deref(), Some("3000,2740"));
}

#[test]
fn should_preserve_bom_across_strip_and_wrap() {
    let original = [0xEF, 0xBB, 0xBF, b'a'];
    let (bom, cuerpo) = sin_bom(&original);
    assert_eq!(bom, BomPresencia::ConBom);
    assert_eq!(con_bom(bom, cuerpo), original.to_vec());
    let (bom, cuerpo) = sin_bom(b"b");
    assert_eq!(bom, BomPresencia::SinBom);
    assert_eq!(con_bom(bom, cuerpo), b"b".to_vec());
}

#[test]
fn should_reject_objects_in_valor_texto() {
    assert!(valor_texto("Model", &json!({"a": 1})).is_err());
    assert_eq!(valor_texto("NbMPPT", &json!(2)).unwrap(), "2");
}

#[test]
fn should_apply_fields_to_respective_objects_in_ond() {
    if !crate::config::compos_ro()
        .join("Inverters")
        .join("Generic_50kW.OND")
        .exists()
    {
        return;
    }
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
