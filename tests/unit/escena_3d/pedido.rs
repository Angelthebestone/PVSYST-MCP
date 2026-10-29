use super::comparar;
use super::describir_escena;
use super::formas::FORMAS;
use super::leer_objetos;
use crate::escena_3d::codec::parse_escena_3d;
use crate::escena_3d::dominio::TipoArchivoEscena;
use serde_json::{json, Value};

/// The scene PVsyst 8.1.8 saved after the editor was driven by the MCP: a 10 x 8 x 3 box at (5, 2, 1).
const CAJA_POSICIONADA: &[u8] =
    include_bytes!("../../fixtures/escena_3d/Casa_B_posicion_8_1_8.SHD");

/// A scene of all 27 shapes with the defaults of PVsyst's dialog, saved by PVsyst 8.1.8.
const FORMAS_GUARDADAS: &[u8] = include_bytes!("../../fixtures/escena_3d/Formas27_8_1_8.SHD");

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

fn error_de(objetos: &Value) -> String {
    describir_escena("Casa_A", objetos)
        .expect_err("invalid request")
        .to_string()
}

#[test]
fn should_describe_the_example_scene_with_the_file_type_of_each_shape() {
    let d = describir_escena("Casa_A", &escena_de_ejemplo()).expect("valid scene");
    let tipos: Vec<&Value> = d["objetos"]
        .as_array()
        .expect("objects")
        .iter()
        .map(|o| &o["forma"]["tipo_pvsyst"])
        .collect();
    assert_eq!(
        tipos,
        [
            &json!("oeParallelepipede"),
            &json!("oeTwo_Sided_Roof_Gables"),
            &json!("oeTree")
        ]
    );
    assert_eq!(d["creado"], false);
    assert_eq!(d["simulado"], true);
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
fn should_list_the_27_shapes_with_unique_keys_and_file_types() {
    let mut claves: Vec<&str> = FORMAS.iter().map(|f| f.tipo).collect();
    let mut tipos: Vec<&str> = FORMAS.iter().map(|f| f.tipo_pvsyst).collect();
    claves.sort_unstable();
    claves.dedup();
    tipos.sort_unstable();
    tipos.dedup();
    assert_eq!((FORMAS.len(), claves.len(), tipos.len()), (27, 27, 27));
}

#[test]
fn should_match_every_shape_with_what_pvsyst_saved_for_its_default_values() {
    let escena = parse_escena_3d(FORMAS_GUARDADAS, TipoArchivoEscena::Shd).expect("saved scene");
    for forma in FORMAS {
        let defectos: Vec<f64> = forma.parametros.iter().map(|p| p.defecto).collect();
        let guardada = escena
            .objetos()
            .iter()
            .filter_map(|o| o.geometria())
            .filter(|g| g.tipo_pvsyst.as_deref() == Some(forma.tipo_pvsyst))
            .any(|g| g.dimensiones == defectos);
        assert!(guardada, "{} no coincide con el archivo", forma.tipo);
    }
}

#[test]
fn should_not_flag_a_roof_that_rests_on_the_box_as_a_collision() {
    let d = describir_escena("Casa_A", &escena_de_ejemplo()).expect("valid scene");
    assert_eq!(d["colisiones"], json!([]));
}

#[test]
fn should_flag_two_boxes_that_share_volume() {
    let objetos = json!([
        {"tipo": "paralelepipedo", "x": 0, "y": 0, "anchura_dx": 10, "longitud_dy": 8, "altura_dz": 3},
        {"tipo": "paralelepipedo", "x": 5, "y": 4, "anchura_dx": 10, "longitud_dy": 8, "altura_dz": 3}
    ]);
    let d = describir_escena("Doble", &objetos).expect("valid scene");
    assert_eq!(d["colisiones"], json!([{"a": 0, "b": 1}]));
    assert!(d["avisos"].to_string().contains("INFERENCIA"));
}

#[test]
fn should_report_every_problem_with_the_index_of_its_object() {
    let e = error_de(&json!([
        {"tipo": "paralelepipedo", "x": 0, "y": 0, "anchura_dx": 10, "longitud_dy": -1, "altura_dz": 3, "color": "rojo"},
        {"tipo": "piscina", "x": 0, "y": 0},
        {"tipo": "arbol", "x": 1, "y": 1, "altura_punto_medio": 6}
    ]));
    assert!(
        e.contains("objetos[0]") && e.contains("'longitud_dy'") && e.contains("fuera de rango"),
        "{e}"
    );
    assert!(e.contains("'color' no existe"), "{e}");
    assert!(e.contains("objetos[1]: tipo 'piscina' desconocido"), "{e}");
    assert!(
        e.contains("objetos[2]") && e.contains("falta 'diametro_tronco'"),
        "{e}"
    );
}

#[test]
fn should_list_the_keys_an_object_accepts_when_one_is_unknown() {
    let e = error_de(
        &json!([{"tipo": "paralelepipedo", "x": 0, "y": 0, "anchura_dx": 1, "longitud_dy": 1, "altura_dz": 1, "hojas": 3}]),
    );
    assert!(e.contains("anchura_dx") && e.contains("altura_dz"), "{e}");
}

#[test]
fn should_require_the_position_of_the_object() {
    let e = error_de(
        &json!([{"tipo": "paralelepipedo", "anchura_dx": 1, "longitud_dy": 1, "altura_dz": 1}]),
    );
    assert!(e.contains("falta 'x'") && e.contains("falta 'y'"), "{e}");
}

#[test]
fn should_reject_non_numbers_and_an_empty_list() {
    let e = error_de(
        &json!([{"tipo": "paralelepipedo", "x": "0", "y": 0, "anchura_dx": 1, "longitud_dy": 1, "altura_dz": 1}]),
    );
    assert!(e.contains("'x' debe ser un número"), "{e}");
    assert!(error_de(&json!([])).contains("al menos un objeto"));
    assert!(error_de(&json!("nada")).contains("lista"));
}

#[test]
fn should_reject_a_scene_name_that_is_not_a_plain_file_name() {
    for nombre in ["", "../Casa", "a/b", "casa\u{0}"] {
        let e = describir_escena(nombre, &escena_de_ejemplo()).expect_err("bad name");
        assert!(e.to_string().contains("'escena'"), "{nombre:?}: {e}");
    }
}

fn pedido(simular: bool) -> Value {
    json!({ "escena": "Casa_A", "objetos": escena_de_ejemplo(), "simular": simular })
}

#[test]
fn should_describe_the_scene_when_simular_is_true() {
    let d = crate::escena_3d::pedir_escena(&pedido(true)).expect("dry run");
    assert_eq!(d["creado"], false);
    assert_eq!(d["escena"], "Casa_A");
}

#[test]
fn should_describe_the_scene_and_say_it_was_not_created_when_simular_is_not_set() {
    let d = crate::escena_3d::pedir_escena(&pedido(false)).expect("validation only");
    assert_eq!(d["creado"], false);
    assert!(
        d["avisos"][0]
            .as_str()
            .is_some_and(|a| a.contains("solo valida el pedido")),
        "{d}"
    );
}

#[test]
fn should_name_the_missing_argument_when_the_request_has_no_objects() {
    let e = crate::escena_3d::pedir_escena(&json!({ "escena": "Casa_A", "simular": true }))
        .expect_err("no objects");
    assert!(e.to_string().contains("objetos"), "{e}");
}

fn caja_en(x: usize) -> Value {
    json!({"tipo": "paralelepipedo", "x": x, "y": 0, "anchura_dx": 1, "longitud_dy": 1, "altura_dz": 1})
}

#[test]
fn should_refuse_a_scene_with_more_objects_than_the_maximum_before_reading_them() {
    let objetos: Vec<Value> = (0..501).map(caja_en).collect();
    let texto = error_de(&Value::Array(objetos));
    assert!(texto.contains("501") && texto.contains("500"), "{texto}");
}

#[test]
fn should_accept_a_scene_with_exactly_the_maximum_of_objects() {
    let objetos: Vec<Value> = (0..500).map(|i| caja_en(i * 2)).collect();
    assert!(describir_escena("Casa_A", &Value::Array(objetos)).is_ok());
}

#[test]
fn should_take_the_default_turn_when_an_optional_value_is_left_out() {
    let objetos = json!([{"tipo": "triangulo", "x": 0, "y": 0,
        "lado_1_2": 3, "lado_2_3": 4, "lado_3_1": 5}]);
    let d = describir_escena("Plano", &objetos).expect("valid scene");
    assert_eq!(d["objetos"][0]["campos"][3]["valor"], json!(0.0));
}

#[test]
fn should_leave_the_shapes_without_a_proven_box_out_of_the_collision_check() {
    let d = describir_escena("Casa_A", &escena_de_ejemplo()).expect("valid scene");
    assert_eq!(d["sin_caja"], json!([1, 2]));
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
fn should_find_every_requested_value_in_the_file_when_the_scene_is_what_was_asked() {
    let estados = estados_de(&json!([{"tipo": "paralelepipedo", "x": 5, "y": 2, "z": 1,
        "anchura_dx": 10, "longitud_dy": 8, "altura_dz": 3}]));
    assert_eq!(estados.len(), 8);
    assert!(estados.iter().all(|(_, e)| e == "\"ok\""), "{estados:?}");
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

#[test]
fn should_report_a_missing_object_when_the_file_has_fewer_than_asked() {
    let dos = json!([
        {"tipo": "paralelepipedo", "x": 0, "y": 0, "anchura_dx": 1, "longitud_dy": 1, "altura_dz": 1},
        {"tipo": "paralelepipedo", "x": 5, "y": 2, "z": 1, "anchura_dx": 10, "longitud_dy": 8, "altura_dz": 3}
    ]);
    let estados = estados_de(&dos);
    assert_eq!(
        estados.last(),
        Some(&("\"objeto\"".into(), "\"falta\"".into()))
    );
}
