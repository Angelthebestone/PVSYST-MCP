//! Behaviour of the prompt templates: argument substitution, defaults and errors.

use super::*;
use serde_json::json;

#[test]
fn should_fill_required_and_default_arguments() {
    let prompt = obtener_prompt(&json!({
        "name": "leer_resultados",
        "arguments": { "proyecto": "Planta" },
    }))
    .expect("known prompt");

    let text = prompt["messages"][0]["content"]["text"]
        .as_str()
        .expect("text");
    assert!(text.contains("Planta"));
    assert!(text.contains("VC0"));
    assert!(!text.contains("{variante}"));
}

#[test]
fn should_accept_numbers_in_prompt_arguments() {
    let prompt = obtener_prompt(&json!({
        "name": "disenar_desde_coordenadas",
        "arguments": { "latitud": 7.07, "longitud": -73.84, "kwp": 10 },
    }))
    .expect("known prompt");

    let text = prompt["messages"][0]["content"]["text"]
        .as_str()
        .expect("text");
    assert!(text.contains("10 kWp"));
    assert!(text.contains("-73.84"));
}

#[test]
fn should_reject_missing_required_argument_with_its_description() {
    let error = obtener_prompt(&json!({
        "name": "disenar_desde_coordenadas",
        "arguments": { "latitud": "7.07", "longitud": "-73.84" },
    }))
    .expect_err("kwp is required");

    assert!(error.to_string().contains("kwp"));
    assert!(error.to_string().contains("Potencia nominal"));
}

#[test]
fn should_reject_unknown_prompt() {
    let error = obtener_prompt(&json!({ "name": "no_existe" })).expect_err("unknown prompt");
    assert!(error.to_string().contains("Prompt desconocido"));
}

#[test]
fn should_keep_ten_recipes_available_as_resource_and_prompts() {
    assert_eq!(como_recurso()["recetas"].as_array().map(Vec::len), Some(10));
    assert_eq!(
        listar_prompts()["prompts"].as_array().map(Vec::len),
        Some(10)
    );
}

#[test]
fn should_expose_the_phase_three_recipes() {
    let prompts = listar_prompts();
    let ids: Vec<&str> = prompts["prompts"]
        .as_array()
        .expect("prompts")
        .iter()
        .filter_map(|prompt| prompt["name"].as_str())
        .collect();
    for id in [
        "convertir_a_seguidor",
        "autoconsumo_con_baterias",
        "comparar_medido",
    ] {
        assert!(ids.contains(&id), "falta la receta {id}: {ids:?}");
    }
}
