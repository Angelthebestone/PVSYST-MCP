//! Behaviour of the prompt templates: argument substitution, defaults and errors.

use super::*;
use serde_json::{json, Value};

#[test]
fn should_fill_required_and_default_arguments() {
    let prompt = obtener_prompt(
        &json!({
            "name": "leer_resultados",
            "arguments": { "proyecto": "Planta" },
        }),
        false,
    )
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
    let prompt = obtener_prompt(
        &json!({
            "name": "disenar_desde_coordenadas",
            "arguments": { "latitud": 7.07, "longitud": -73.84, "kwp": 10 },
        }),
        false,
    )
    .expect("known prompt");

    let text = prompt["messages"][0]["content"]["text"]
        .as_str()
        .expect("text");
    assert!(text.contains("10 kWp"));
    assert!(text.contains("-73.84"));
}

#[test]
fn should_reject_missing_required_argument_with_its_description() {
    let error = obtener_prompt(
        &json!({
            "name": "disenar_desde_coordenadas",
            "arguments": { "latitud": "7.07", "longitud": "-73.84" },
        }),
        false,
    )
    .expect_err("kwp is required");

    assert!(error.to_string().contains("kwp"));
    assert!(error.to_string().contains("Potencia nominal"));
}

#[test]
fn should_reject_unknown_prompt() {
    let error = obtener_prompt(&json!({ "name": "no_existe" }), false).expect_err("unknown prompt");
    assert!(error.to_string().contains("Prompt desconocido"));
}

#[test]
fn should_keep_nine_native_recipes_available_without_approximations() {
    assert_eq!(
        como_recurso(false)["recetas"].as_array().map(Vec::len),
        Some(9)
    );
    assert_eq!(
        listar_prompts(false)["prompts"].as_array().map(Vec::len),
        Some(9)
    );
    assert_eq!(
        como_recurso(true)["recetas"].as_array().map(Vec::len),
        Some(10)
    );
    assert_eq!(
        listar_prompts(true)["prompts"].as_array().map(Vec::len),
        Some(10)
    );
}

#[test]
fn should_hide_approximation_recipes_without_the_flag() {
    let ids = |value: Value| -> Vec<String> {
        value["prompts"]
            .as_array()
            .expect("prompts")
            .iter()
            .filter_map(|p| p["name"].as_str().map(String::from))
            .collect()
    };
    let sin = ids(listar_prompts(false));
    assert!(sin.contains(&"optimizar_orientacion".to_string()));
    assert!(!sin.contains(&"comparar_medido".to_string()));

    let con = ids(listar_prompts(true));
    assert!(con.contains(&"optimizar_orientacion".to_string()));
    assert!(con.contains(&"comparar_medido".to_string()));
}

#[test]
fn should_expose_the_phase_three_recipes() {
    let prompts = listar_prompts(true);
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

#[test]
fn should_give_disenar_desde_coordenadas_component_names_that_the_bundle_ships() {
    let prompt = obtener_prompt(
        &json!({
            "name": "disenar_desde_coordenadas",
            "arguments": { "latitud": "7.07", "longitud": "-73.84", "kwp": "10" },
        }),
        false,
    )
    .expect("known prompt");
    let text = prompt["messages"][0]["content"]["text"]
        .as_str()
        .expect("text");
    assert!(text.contains("Generic_Mono_300W.PAN"), "{text}");
    assert!(text.contains("Generic_7.5kW.OND"), "{text}");
    assert!(!text.contains("PVSYST_MCP_APROXIMACIONES"), "{text}");
}
