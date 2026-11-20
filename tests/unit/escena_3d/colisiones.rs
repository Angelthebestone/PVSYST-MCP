use super::super::describir_escena;
use super::super::informe::MAX_COLISIONES_LISTADAS;
use serde_json::{json, Value};

#[test]
fn should_bound_the_listed_collisions_when_hundreds_of_boxes_overlap() {
    let caja = json!({"tipo": "paralelepipedo", "x": 0, "y": 0, "anchura_dx": 10, "longitud_dy": 8, "altura_dz": 3});
    let objetos = Value::Array(vec![caja; 500]);

    let d = describir_escena("Solapadas", &objetos).expect("valid scene");

    assert_eq!(d["colisiones_total"], json!(500 * 499 / 2));
    assert_eq!(
        d["colisiones"].as_array().map(Vec::len),
        Some(MAX_COLISIONES_LISTADAS)
    );
}
