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
        Idioma::En,
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
        Idioma::En,
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
        Idioma::En,
    )
    .expect_err("kwp is required");

    assert!(error.to_string().contains("kwp"));
    assert!(error.to_string().contains("Nominal power"));
}

#[test]
fn should_reject_unknown_prompt() {
    let error = obtener_prompt(&json!({ "name": "no_existe" }), false, Idioma::En)
        .expect_err("unknown prompt");
    assert!(error.to_string().contains("Unknown prompt"));
}

#[test]
fn should_keep_nine_native_recipes_available_without_approximations() {
    assert_eq!(
        como_recurso(false, Idioma::En)["recetas"]
            .as_array()
            .map(Vec::len),
        Some(9)
    );
    assert_eq!(
        listar_prompts(false, Idioma::En)["prompts"]
            .as_array()
            .map(Vec::len),
        Some(9)
    );
    assert_eq!(
        como_recurso(true, Idioma::En)["recetas"]
            .as_array()
            .map(Vec::len),
        Some(10)
    );
    assert_eq!(
        listar_prompts(true, Idioma::En)["prompts"]
            .as_array()
            .map(Vec::len),
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
    let sin = ids(listar_prompts(false, Idioma::En));
    assert!(sin.contains(&"optimizar_orientacion".to_string()));
    assert!(!sin.contains(&"comparar_medido".to_string()));

    let con = ids(listar_prompts(true, Idioma::En));
    assert!(con.contains(&"optimizar_orientacion".to_string()));
    assert!(con.contains(&"comparar_medido".to_string()));
}

#[test]
fn should_expose_the_phase_three_recipes() {
    let prompts = listar_prompts(true, Idioma::En);
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
fn should_point_disenar_desde_coordenadas_at_disenar_sistema_without_invented_components() {
    let prompt = obtener_prompt(
        &json!({
            "name": "disenar_desde_coordenadas",
            "arguments": { "latitud": "7.07", "longitud": "-73.84", "kwp": "10" },
        }),
        false,
        Idioma::En,
    )
    .expect("known prompt");
    let text = prompt["messages"][0]["content"]["text"]
        .as_str()
        .expect("text");
    assert!(text.contains("disenar_sistema"), "{text}");
    assert!(!text.contains("Generic_"), "{text}");
    assert!(!text.contains("PVSYST_MCP_APROXIMACIONES"), "{text}");
}

#[test]
fn should_serve_recipes_in_spanish_when_the_language_is_es() {
    let recurso = como_recurso(false, Idioma::Es);
    assert_eq!(
        recurso["intro"].as_str().unwrap(),
        "Recetas listas para usar. Cada una indica cuándo usarla y la secuencia de tools."
    );

    let prompts = listar_prompts(false, Idioma::Es);
    let leer = prompts["prompts"]
        .as_array()
        .expect("prompts")
        .iter()
        .find(|p| p["name"] == "leer_resultados")
        .expect("leer_resultados prompt");
    assert_eq!(leer["title"].as_str().unwrap(), "Leer resultados");

    let prompt = obtener_prompt(
        &json!({
            "name": "disenar_desde_coordenadas",
            "arguments": { "latitud": "7.07", "longitud": "-73.84" },
        }),
        false,
        Idioma::Es,
    )
    .expect_err("kwp is required");
    assert!(prompt.to_string().contains("Potencia nominal en kWp"));
}
