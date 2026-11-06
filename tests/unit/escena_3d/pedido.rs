use super::comparar;
use super::describir_escena;
use super::leer_objetos;
use crate::escena_3d::codec::parse_escena_3d;
use crate::escena_3d::dominio::TipoArchivoEscena;
use serde_json::{json, Value};

mod colisiones;

/// The scene PVsyst 8.1.8 saved after the editor was driven by the MCP: a 10 x 8 x 3 box at (5, 2, 1).
const CAJA_POSICIONADA: &[u8] =
    include_bytes!("../../fixtures/escena_3d/Casa_B_posicion_8_1_8.SHD");

/// Shapes 15-27 built with a different value in every parameter (13 objects), and the scene PVsyst 8.1.8 saved for them.
const FORMAS_DISTINTAS_PEDIDO: &str =
    include_str!("../../fixtures/escena_3d/Formas13_distintos_pedido.json");
const FORMAS_DISTINTAS_GUARDADAS: &[u8] =
    include_bytes!("../../fixtures/escena_3d/Formas13_distintos_8_1_8.SHD");

fn escena_de_ejemplo() -> Value {
    json!([
        {"tipo": "paralelepipedo", "x": 0, "y": 0, "z": 0, "anchura_dx": 10, "longitud_dy": 8, "altura_dz": 3},
        {"tipo": "techo_dos_aguas_hastial", "x": 0, "y": 0, "z": 3, "ancho_base": 8, "longitud_superior": 10,
         "inclinacion_techo_deg": 25, "aguilon_1_angulo_deg": 0, "aguilon_2_angulo_deg": 0},
        {"tipo": "arbol", "x": 15, "y": 2, "altura_punto_medio": 6, "altura_media": 5, "altura_parte_baja": 3,
         "altura_tronco": 2, "diametro_medio": 4, "diametro_tronco": 0.4}
    ])
}

#[test]
fn should_give_each_dimension_the_position_it_has_in_params() {
    let d = describir_escena("Casa_A", &escena_de_ejemplo()).expect("valid scene");
    let posiciones: Vec<(&Value, &Value)> = d["objetos"][1]["campos"]
        .as_array()
        .expect("fields")
        .iter()
        .take(3)
        .map(|c| (&c["posicion_en_params"], &c["valor"]))
        .collect();
    assert_eq!(
        posiciones,
        [
            (&json!(0), &json!(8.0)),
            (&json!(1), &json!(10.0)),
            (&json!(2), &json!(25.0))
        ]
    );
}

#[test]
fn should_reject_a_scene_name_that_is_not_a_plain_file_name() {
    for nombre in ["", "../Casa", "a/b", "casa\u{0}"] {
        let e = describir_escena(nombre, &escena_de_ejemplo()).expect_err("bad name");
        assert!(e.to_string().contains("'escena'"), "{nombre:?}: {e}");
    }
}

#[test]
fn should_find_every_value_of_shapes_15_to_27_in_what_pvsyst_saved() {
    let pedido: Value = serde_json::from_str(FORMAS_DISTINTAS_PEDIDO).expect("request fixture");
    let pedidos = leer_objetos(&pedido).expect("valid request");
    let escena =
        parse_escena_3d(FORMAS_DISTINTAS_GUARDADAS, TipoArchivoEscena::Shd).expect("saved scene");
    let filas = comparar(&escena, &pedidos);
    assert_eq!(pedidos.len(), 13);
    assert!(!filas.is_empty());
    assert!(filas.iter().all(|f| f["estado"] == "ok"), "{filas:?}");
}

fn estados_de(pedido: &Value) -> Vec<(String, String)> {
    let pedidos = leer_objetos(pedido).expect("valid request");
    let escena = parse_escena_3d(CAJA_POSICIONADA, TipoArchivoEscena::Shd).expect("saved scene");
    comparar(&escena, &pedidos)
        .iter()
        .map(|c| (c["clave"].to_string(), c["estado"].to_string()))
        .collect()
}

#[test]
fn should_name_the_values_that_differ_from_the_file() {
    let estados = estados_de(&json!([{"tipo": "paralelepipedo", "x": 5, "y": 3, "z": 1,
        "anchura_dx": 10, "longitud_dy": 9, "altura_dz": 3}]));
    let difieren: Vec<&str> = estados
        .iter()
        .filter(|(_, e)| e != "\"ok\"")
        .map(|(k, _)| k.as_str())
        .collect();
    assert_eq!(difieren, ["\"longitud_dy\"", "\"y\""]);
}
