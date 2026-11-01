use super::{decidir_marca, Marca};
use crate::error::Error;
use crate::vista_agente::campos_vivos::DESCONOCIDO;
use crate::vista_agente::esquemas::entidad;
use crate::vista_agente::plan::planificar;
use serde_json::{json, Map, Value};

fn valores(v: &Value) -> Map<String, Value> {
    v.as_object().cloned().unwrap_or_default()
}

#[test]
fn should_not_click_when_the_box_already_reads_as_asked() {
    assert_eq!(decidir_marca("true", "true"), Marca::YaEsta);
    assert_eq!(decidir_marca("false", "false"), Marca::YaEsta);
}

#[test]
fn should_click_when_the_box_reads_the_opposite() {
    assert_eq!(decidir_marca("false", "true"), Marca::Pulsar);
    assert_eq!(decidir_marca("true", "false"), Marca::Pulsar);
}

#[test]
fn should_not_click_when_the_state_of_the_box_is_unknown() {
    assert_eq!(decidir_marca(DESCONOCIDO, "true"), Marca::SinLectura);
    assert_eq!(decidir_marca(DESCONOCIDO, "false"), Marca::SinLectura);
}

#[test]
fn should_reject_the_whole_plan_when_an_option_is_asked_off() {
    let sistema = entidad("variante.system").expect("system schema");

    let plan = planificar(
        sistema,
        &valores(&json!({ "usar_potencia_planeada": false, "n_cadenas": 4 })),
    );

    assert!(matches!(&plan, Err(Error::Invalid(m)) if m.contains("usar_potencia_planeada")));
}

#[test]
fn should_accept_an_option_asked_on_and_a_check_box_asked_off() {
    let sistema = entidad("variante.system").expect("system schema");
    let inversor = entidad("components.inverters").expect("inverter schema");

    let opcion = planificar(
        sistema,
        &valores(&json!({ "usar_potencia_planeada": true })),
    );
    let casilla = planificar(
        inversor,
        &valores(&json!({ "definir_corriente_ca_nominal": false })),
    );

    assert!(opcion.is_ok());
    assert!(casilla.is_ok());
}
