use super::campos_vivos::{localizar, CampoVivo};
use super::dominio::{Ancla, TOLERANCIA_ANCLA_PX};
use super::esquemas::{de_formulario, entidad, todas};
use super::plan::{coincide, es_valido, planificar};
use crate::error::Error;
use serde_json::{json, Map, Value};

mod coincidencia;
mod guardados;

fn valores(v: &Value) -> Map<String, Value> {
    v.as_object().cloned().unwrap_or_default()
}

fn panel() -> &'static super::dominio::Entidad {
    entidad("components.panels").expect("panel schema")
}

fn vivo(clase: &str, x: i32, y: i32) -> CampoVivo {
    CampoVivo {
        hwnd: 1,
        clase: clase.into(),
        texto: String::new(),
        habilitado: true,
        posicion: Ancla { x, y },
    }
}

#[test]
fn should_plan_changes_in_schema_order_when_values_are_valid() {
    let cambios = planificar(
        panel(),
        &valores(&json!({"voc_v": 49.0, "modelo": "X1", "potencia_nominal_w": 200})),
    )
    .expect("plan");
    let claves: Vec<&str> = cambios.iter().map(|c| c.campo.clave).collect();
    assert_eq!(claves, ["modelo", "potencia_nominal_w", "voc_v"]);
    assert_eq!(cambios[1].texto, "200");
}

#[test]
fn should_report_every_problem_when_several_values_are_wrong() {
    let e = planificar(
        panel(),
        &valores(&json!({"potencia_nominal_w": 99999, "no_existe": 1, "t_oper_c": 20})),
    )
    .expect_err("invalid")
    .to_string();
    assert!(e.contains("fuera de rango"), "{e}");
    assert!(e.contains("'no_existe' no existe"), "{e}");
    assert!(e.contains("'t_oper_c' es de solo lectura"), "{e}");
    assert!(
        e.contains("potencia_nominal_w"),
        "valid keys are listed: {e}"
    );
}

#[test]
fn should_accept_numbers_given_as_text_with_a_comma() {
    let cambios = planificar(panel(), &valores(&json!({"isc_a": "10,5"}))).expect("plan");
    assert_eq!(cambios[0].texto, "10.5");
}

#[test]
fn should_reject_fractional_values_for_integer_fields() {
    let e = planificar(panel(), &valores(&json!({"celulas_en_serie": 72.5}))).expect_err("invalid");
    assert!(e.to_string().contains("entero"), "{e}");
}

#[test]
fn should_reject_an_empty_request() {
    let err = planificar(panel(), &Map::new()).unwrap_err();
    assert!(
        matches!(&err, Error::Invalid(msg) if msg.contains("no hay ningún valor")),
        "error inesperado para petición vacía: {err}"
    );
}

#[test]
fn should_match_numbers_by_value_when_pvsyst_reformats_them() {
    let campo = panel().campo("potencia_nominal_w").expect("field");
    assert!(coincide(campo, "200", "200.0"));
    assert!(!coincide(campo, "200", "201"));
}

#[test]
fn should_flag_a_field_value_outside_its_range() {
    let campo = panel().campo("potencia_nominal_w").expect("field");
    assert!(es_valido(campo, "0.1"));
    assert!(!es_valido(campo, "N/A"));
    assert!(!es_valido(campo, "5000"));
}

#[test]
fn should_locate_the_control_nearest_to_the_anchor_of_its_class() {
    let campo = panel().campo("isc_a").expect("field");
    let vivos = [
        vivo("TEdit", campo.ancla.x + 30, campo.ancla.y),
        vivo("TEdit", campo.ancla.x + 2, campo.ancla.y - 1),
        vivo("TComboBox", campo.ancla.x, campo.ancla.y),
    ];
    let hallado = localizar(campo, &vivos).expect("found");
    assert_eq!(hallado.posicion.x, campo.ancla.x + 2);
    assert_eq!(hallado.clase, "TEdit");
}

#[test]
fn should_not_locate_a_control_beyond_the_tolerance() {
    let campo = panel().campo("isc_a").expect("field");
    let lejos = [vivo(
        "TEdit",
        campo.ancla.x + TOLERANCIA_ANCLA_PX + 1,
        campo.ancla.y,
    )];
    assert!(localizar(campo, &lejos).is_none());
}

fn verificar_pares_de_campos(
    entidad_clave: &str,
    campos: &[crate::vista_agente::dominio::CampoEsquema],
) {
    for (i, a) in campos.iter().enumerate() {
        for b in &campos[i + 1..] {
            assert_ne!(a.clave, b.clave, "duplicate key in {entidad_clave}");
            let cerca = (a.ancla.x - b.ancla.x).abs() <= TOLERANCIA_ANCLA_PX
                && (a.ancla.y - b.ancla.y).abs() <= TOLERANCIA_ANCLA_PX;
            assert!(
                !(cerca && a.clase == b.clase),
                "{} and {} share an anchor",
                a.clave,
                b.clave
            );
        }
    }
}

#[test]
fn should_keep_schema_keys_and_anchors_unambiguous() {
    for e in todas() {
        verificar_pares_de_campos(e.clave, e.campos);
    }
}

#[test]
fn should_resolve_an_entity_from_its_window_class() {
    assert_eq!(
        de_formulario("TPVModuleForm").map(|e| e.clave),
        Some("components.panels")
    );
    assert!(de_formulario("TNoExiste").is_none());
}

#[test]
fn should_reject_an_unknown_schema_entity_instead_of_listing_all() {
    let r = crate::vista_agente::listar_esquemas(&json!({ "entidad": "zzz" }));
    assert!(matches!(r, Err(crate::error::Error::Invalid(_))));
}

#[test]
fn should_return_only_the_requested_schema_entity() {
    let v = crate::vista_agente::listar_esquemas(&json!({ "entidad": "components.panels" }))
        .expect("schema");
    assert_eq!(v["count"], json!(1));
}

#[test]
fn should_put_only_the_efficiency_fields_of_the_inverter_on_the_second_tab() {
    let esquema = entidad("components.inverters").expect("inverter schema");
    let en_pagina_1: Vec<&str> = esquema
        .campos
        .iter()
        .filter(|c| c.pagina == 1)
        .map(|c| c.clave)
        .collect();
    assert_eq!(en_pagina_1, ["eficiencia_max_pct", "eficiencia_euro_pct"]);
    assert!(panel().campos.iter().all(|c| c.pagina == 0));
}

#[test]
fn should_describe_the_27_shapes_of_the_3d_editor_when_their_entity_is_asked_for() {
    let v = crate::vista_agente::listar_esquemas(&json!({ "entidad": "escena_3d.formas" }))
        .expect("shape schema");
    let formas = v["formas"].as_array().expect("shapes");
    assert_eq!(formas.len(), 27);
    let tree = formas.iter().find(|f| f["tipo"] == "arbol").expect("tree");
    assert_eq!(tree["tipo_pvsyst"], "oeTree");
    assert_eq!(tree["campos"][0]["unidad"], "m");
    let comunes: Vec<&str> = v["comunes"]
        .as_array()
        .expect("common keys")
        .iter()
        .map(|c| c["clave"].as_str().unwrap_or(""))
        .collect();
    assert!(comunes.contains(&"rotacion_azimut_deg"));
}

#[test]
fn should_point_to_the_shape_entity_in_the_unfiltered_schema_listing() {
    let v = crate::vista_agente::listar_esquemas(&json!({})).expect("listing");
    assert_eq!(v["otras_entidades"][0], "escena_3d.formas");
}

#[test]
fn should_accept_true_or_false_for_an_option_and_reject_anything_else() {
    let sistema = entidad("variante.system").expect("system schema");

    let bien = planificar(
        sistema,
        &valores(&json!({ "usar_potencia_planeada": true })),
    );
    let mal = planificar(
        sistema,
        &valores(&json!({ "usar_potencia_planeada": "si" })),
    );

    assert_eq!(bien.expect("plan")[0].texto, "true");
    assert!(matches!(mal, Err(Error::Invalid(_))));
}

#[test]
fn should_list_the_option_as_a_checkbox_type_and_keep_its_schema_unambiguous() {
    let sistema = entidad("variante.system").expect("system schema");

    let tipo = sistema.campo("usar_area_disponible").expect("option").tipo;

    assert_eq!(tipo.nombre(), "casilla");
}

/// Fields that depend on a check box, with the type of the check box declared before them.
fn tipos_de_casilla_anterior() -> Vec<(&'static str, Option<&'static str>)> {
    todas()
        .iter()
        .flat_map(|e| {
            e.campos.iter().enumerate().filter_map(move |(i, campo)| {
                let casilla = campo.depende_de?;
                let anterior = e.campos.iter().take(i).find(|c| c.clave == casilla);
                Some((campo.clave, anterior.map(|c| c.tipo.nombre())))
            })
        })
        .collect()
}

#[test]
fn should_declare_a_check_box_before_every_field_that_depends_on_it() {
    let dependientes = tipos_de_casilla_anterior();

    assert!(!dependientes.is_empty());
    for (clave, tipo) in dependientes {
        assert_eq!(
            tipo,
            Some("casilla"),
            "{clave}: su casilla no está declarada antes"
        );
    }
}

#[test]
fn should_tell_which_check_box_a_conditional_inverter_field_needs_when_the_schema_is_listed() {
    let listado =
        crate::vista_agente::listar_esquemas(&json!({ "entidad": "components.inverters" }))
            .expect("listing");

    let campos = listado["entidades"][0]["campos"]
        .as_array()
        .expect("fields");
    let corriente = campos
        .iter()
        .find(|c| c["clave"] == "corriente_ca_nominal_a")
        .expect("field");

    assert_eq!(corriente["depende_de"], "definir_corriente_ca_nominal");
}
