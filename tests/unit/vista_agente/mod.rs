use super::campos_vivos::{localizar, CampoVivo};
use super::dominio::{Ancla, TOLERANCIA_ANCLA_PX};
use super::esquemas::{de_formulario, entidad, todas};
use super::plan::{coincide, es_valido, planificar};
use crate::error::Error;
use serde_json::{json, Map, Value};

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
