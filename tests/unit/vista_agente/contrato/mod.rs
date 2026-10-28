use super::campos::unidad_de;
use super::catalogo::todas;
use super::construccion::{construir, Contexto};
use super::tipos::{Campo, ErrorVista, Estado, Origen, PasoSiguiente, Recorte, Vista};
use serde_json::{json, Value};

fn vista_vacia() -> Vista {
    Vista {
        estado: Estado::Ok,
        resumen: "hecho".into(),
        origen: Origen::Pvsyst,
        campos: Vec::new(),
        avisos: Vec::new(),
        siguiente: Vec::new(),
        recortado: Vec::new(),
        error: None,
    }
}

fn campo(clave: &str) -> Campo {
    Campo {
        clave: clave.into(),
        valor: json!(1),
        unidad: None,
        editable: false,
    }
}

fn construir_de(tool: &str, accion: &str, salida: &Value) -> Vista {
    let args = json!({});
    construir(
        &Contexto {
            tool,
            accion,
            args: &args,
        },
        salida,
    )
}

#[test]
fn should_reject_a_view_with_repeated_field_keys() {
    let mut vista = vista_vacia();
    vista.campos = vec![campo("kwp"), campo("kwp")];
    let motivo = vista.validar(&[]).expect_err("repeated key");
    assert!(motivo.contains("kwp"), "{motivo}");
}

#[test]
fn should_reject_a_view_that_cut_a_protected_key() {
    let mut vista = vista_vacia();
    vista.recortado = vec![Recorte {
        clave: "campos".into(),
        mostrados: 1,
        total: 2,
        pedir_resto: "detalle:completo",
    }];
    assert!(vista.validar(&["campos", "botones"]).is_err());
}

#[test]
fn should_reject_an_error_state_without_the_error_block() {
    let mut vista = vista_vacia();
    vista.estado = Estado::Error;
    assert!(vista.validar(&[]).is_err());
    vista.error = Some(ErrorVista {
        codigo: "invalid".into(),
        motivo: "x".into(),
        causa_probable: "y",
        que_hacer: "z",
        claves_validas: Vec::new(),
    });
    assert!(vista.validar(&[]).is_ok());
}

#[test]
fn should_reject_a_next_step_whose_arguments_are_not_an_object() {
    let mut vista = vista_vacia();
    vista.siguiente = vec![PasoSiguiente {
        tool: "listar".into(),
        argumentos: json!("proyectos"),
        motivo: "x",
    }];
    assert!(vista.validar(&[]).is_err());
}

#[test]
fn should_not_repeat_a_catalogue_entry() {
    let mut vistas = std::collections::HashSet::new();
    let repetida = todas().find(|e| !vistas.insert((e.tool, e.accion)));
    assert!(repetida.is_none(), "{repetida:?}");
}

#[test]
fn should_give_every_catalogue_entry_a_summary_and_a_next_step_that_needs_no_value() {
    for e in todas() {
        assert!(!e.resumen.is_empty(), "{} {}", e.tool, e.accion);
        assert!(
            e.siguiente.iter().any(|p| p.tomar.is_empty()),
            "{} {}: ningún paso es seguro sin valores",
            e.tool,
            e.accion
        );
    }
}

#[test]
fn should_read_the_unit_from_the_last_segment_of_the_key() {
    assert_eq!(
        unidad_de("site.annual_ghi_kwh_m2").as_deref(),
        Some("kWh/m²")
    );
    assert_eq!(unidad_de("potencia_nominal_w").as_deref(), Some("W"));
    assert_eq!(unidad_de("modelo"), None);
}

#[test]
fn should_flatten_nested_scalars_with_dotted_keys_and_skip_long_texts() {
    let salida = json!({
        "tipo_sistema": "red",
        "ubicacion": { "latitud": 40.4 },
        "log": "x".repeat(500),
        "lista": [1, 2],
    });
    let claves: Vec<String> = construir_de("sitio", "crear", &salida)
        .campos
        .into_iter()
        .map(|c| c.clave)
        .collect();
    assert_eq!(claves, ["tipo_sistema", "ubicacion.latitud"]);
}

#[test]
fn should_gather_warnings_from_every_spelling_without_repeating() {
    let salida = json!({ "avisos": ["a", "b"], "advertencias": ["a"], "aviso": "c" });
    let avisos = construir_de("listar", "proyectos", &salida).avisos;
    assert_eq!(avisos, ["a", "b", "c"]);
}

#[test]
fn should_keep_the_ok_state_when_the_answer_only_has_trimmed_lists() {
    let salida = json!({ "items": [1], "items_total": 20, "truncado": true });
    let vista = construir_de("listar", "archivos", &salida);
    assert_eq!(vista.estado, Estado::Ok);
    assert_eq!(vista.recortado.len(), 1);
}

#[test]
fn should_serialize_with_stable_names() {
    let valor = serde_json::to_value(vista_vacia()).expect("serializable");
    assert_eq!(valor["estado"], "ok");
    assert!(valor.get("avisos").is_none(), "empty lists are not sent");
    assert!(valor["campos"].is_array() && valor["siguiente"].is_array());
}

#[test]
fn should_list_only_the_first_warnings_and_say_how_many_were_left_out() {
    let avisos: Vec<String> = (0..9).map(|i| format!("aviso {i}")).collect();
    let vista = construir_de("proyecto", "doctor", &json!({ "avisos": avisos }));
    assert_eq!(vista.avisos.len(), 5);
    let recorte = vista.recortado.first().expect("cut reported");
    assert_eq!((recorte.clave.as_str(), recorte.total), ("vista.avisos", 9));
}

#[test]
fn should_skip_a_nested_object_that_is_a_table_rather_than_a_group_of_values() {
    let tabla: serde_json::Map<String, Value> =
        (0..20).map(|i| (format!("fila_{i}"), json!(i))).collect();
    let salida = json!({ "secciones": tabla, "count": 20 });
    let claves: Vec<String> = construir_de("entorno", "preferencias", &salida)
        .campos
        .into_iter()
        .map(|c| c.clave)
        .collect();
    assert_eq!(claves, ["count"]);
}
