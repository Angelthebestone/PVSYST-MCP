use super::describir_escena;
use serde_json::{json, Value};

fn escena_de_ejemplo() -> Value {
    json!([
        {"tipo": "paralelepipedo", "x": 0, "y": 0, "z": 0, "largo": 10, "ancho": 8, "alto": 3},
        {"tipo": "tejado_dos_aguas", "x": 0, "y": 0, "z": 3, "largo": 10, "ancho": 8, "inclinacion_deg": 25},
        {"tipo": "arbol", "x": 15, "y": 2, "radio_copa": 2, "radio_tronco": 0.2, "altura": 6}
    ])
}

fn error_de(objetos: &Value) -> String {
    describir_escena("Casa_A", objetos)
        .expect_err("invalid request")
        .to_string()
}

#[test]
fn should_describe_the_example_scene_with_the_file_code_of_each_shape() {
    let d = describir_escena("Casa_A", &escena_de_ejemplo()).expect("valid scene");
    let codigos: Vec<&Value> = d["objetos"]
        .as_array()
        .expect("objects")
        .iter()
        .map(|o| &o["forma"]["codigo"])
        .collect();
    assert_eq!(codigos, [&json!(11), &json!(23), &json!(21)]);
    assert_eq!(d["creado"], false);
    assert_eq!(d["simulado"], true);
}

#[test]
fn should_place_each_dimension_in_the_slot_the_file_format_uses() {
    let d = describir_escena("Casa_A", &escena_de_ejemplo()).expect("valid scene");
    let ranuras: Vec<(&Value, &Value)> = d["objetos"][1]["ranuras_archivo"]
        .as_array()
        .expect("slots")
        .iter()
        .map(|r| (&r["ranura"], &r["valor"]))
        .collect();
    assert_eq!(
        ranuras,
        [
            (&json!(0), &json!(10.0)),
            (&json!(1), &json!(8.0)),
            (&json!(2), &json!(25.0))
        ]
    );
}

#[test]
fn should_not_flag_a_roof_that_rests_on_the_box_as_a_collision() {
    let d = describir_escena("Casa_A", &escena_de_ejemplo()).expect("valid scene");
    assert_eq!(d["colisiones"], json!([]));
}

#[test]
fn should_flag_two_boxes_that_share_volume() {
    let objetos = json!([
        {"tipo": "paralelepipedo", "x": 0, "y": 0, "largo": 10, "ancho": 8, "alto": 3},
        {"tipo": "paralelepipedo", "x": 5, "y": 4, "largo": 10, "ancho": 8, "alto": 3}
    ]);
    let d = describir_escena("Doble", &objetos).expect("valid scene");
    assert_eq!(d["colisiones"], json!([{"a": 0, "b": 1}]));
    assert!(d["avisos"].to_string().contains("INFERENCIA"));
}

#[test]
fn should_report_every_problem_with_the_index_of_its_object() {
    let e = error_de(&json!([
        {"tipo": "paralelepipedo", "x": 0, "y": 0, "largo": 10, "ancho": -1, "alto": 3, "color": "rojo"},
        {"tipo": "piscina", "x": 0, "y": 0},
        {"tipo": "arbol", "x": 1, "y": 1, "radio_copa": 1, "radio_tronco": 2, "altura": 5}
    ]));
    assert!(
        e.contains("objetos[0]") && e.contains("'ancho'") && e.contains("fuera de rango"),
        "{e}"
    );
    assert!(e.contains("'color' no existe"), "{e}");
    assert!(e.contains("objetos[1]: tipo 'piscina' desconocido"), "{e}");
    assert!(
        e.contains("objetos[2]") && e.contains("radio_tronco"),
        "{e}"
    );
}

#[test]
fn should_list_the_keys_an_object_accepts_when_one_is_unknown() {
    let e = error_de(
        &json!([{"tipo": "arbol", "x": 0, "y": 0, "radio_copa": 2, "radio_tronco": 0.2, "altura": 6, "hojas": 3}]),
    );
    assert!(e.contains("radio_copa") && e.contains("altura"), "{e}");
}

#[test]
fn should_require_the_position_of_the_object() {
    let e = error_de(&json!([{"tipo": "paralelepipedo", "largo": 1, "ancho": 1, "alto": 1}]));
    assert!(e.contains("falta 'x'") && e.contains("falta 'y'"), "{e}");
}

#[test]
fn should_reject_non_numbers_and_an_empty_list() {
    let e = error_de(
        &json!([{"tipo": "paralelepipedo", "x": "0", "y": 0, "largo": 1, "ancho": 1, "alto": 1}]),
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
fn should_refuse_to_create_the_scene_when_simular_is_not_set() {
    let e = crate::escena_3d::pedir_escena(&pedido(false)).expect_err("not implemented");
    assert!(e.to_string().contains("simular:true"), "{e}");
}

#[test]
fn should_name_the_missing_argument_when_the_request_has_no_objects() {
    let e = crate::escena_3d::pedir_escena(&json!({ "escena": "Casa_A", "simular": true }))
        .expect_err("no objects");
    assert!(e.to_string().contains("objetos"), "{e}");
}
