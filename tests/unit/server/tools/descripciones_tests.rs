use super::*;
use crate::config::Paths;
use serde_json::Value;

/// Arguments of the schema a client sees that still carry no description. It only
/// goes down: a new argument is described in its tool's table, and each table that
/// gets filled lowers this number until it reaches zero.
const DEUDA_MAXIMA: usize = 133;

/// Longest description, in characters: the tool list travels in every turn, so a
/// description says the meaning and the unit, not a paragraph.
const LARGO_MAXIMO: usize = 100;

fn esquemas(idioma: Idioma, aproximaciones: bool) -> Vec<Value> {
    let mut paths = Paths::para_test(std::env::temp_dir());
    paths.idioma = idioma;
    paths.aproximaciones = aproximaciones;
    super::super::list(&paths)
}

fn propiedades(esquema: &Value) -> Vec<(&String, &Value)> {
    esquema
        .pointer("/inputSchema/properties")
        .and_then(Value::as_object)
        .map(|m| m.iter().collect())
        .unwrap_or_default()
}

fn nombre(esquema: &Value) -> &str {
    esquema
        .get("name")
        .and_then(Value::as_str)
        .unwrap_or_default()
}

#[test]
fn should_not_add_arguments_without_a_description() {
    let esquemas = esquemas(Idioma::En, false);
    let sin: Vec<String> = esquemas
        .iter()
        .flat_map(|e| {
            propiedades(e)
                .into_iter()
                .filter(|(_, v)| v.get("description").is_none())
                .map(|(clave, _)| format!("{}.{clave}", nombre(e)))
        })
        .collect();
    assert!(
        sin.len() <= DEUDA_MAXIMA,
        "{} arguments have no description (limit {DEUDA_MAXIMA}); describe the new one in its tool's table under src/server/tools/descripciones: {sin:?}",
        sin.len()
    );
}

#[test]
fn should_describe_in_every_tool_only_arguments_it_declares() {
    for esquema in esquemas(Idioma::En, true) {
        let declaradas: Vec<&String> = propiedades(&esquema)
            .into_iter()
            .map(|(clave, _)| clave)
            .collect();
        for entrada in tabla(nombre(&esquema)) {
            assert!(
                declaradas.iter().any(|d| d.as_str() == entrada.clave),
                "{} describes '{}', which the tool does not declare",
                nombre(&esquema),
                entrada.clave
            );
        }
    }
}

/// Keys written twice in one table: the second one would never be read.
fn repetidas(entradas: &[Entrada]) -> Vec<&'static str> {
    entradas
        .iter()
        .enumerate()
        .filter(|(i, e)| entradas.iter().skip(i + 1).any(|o| o.clave == e.clave))
        .map(|(_, e)| e.clave)
        .collect()
}

#[test]
fn should_describe_each_key_once_and_in_both_languages() {
    for esquema in esquemas(Idioma::En, true) {
        let propias = tabla(nombre(&esquema));
        assert_eq!(
            repetidas(propias),
            Vec::<&str>::new(),
            "{} describes a key twice",
            nombre(&esquema)
        );
        let sin_idioma: Vec<&str> = propias
            .iter()
            .chain(comunes::TABLA)
            .filter(|e| e.texto.es.trim().is_empty() || e.texto.en.trim().is_empty())
            .map(|e| e.clave)
            .collect();
        assert!(sin_idioma.is_empty(), "{sin_idioma:?} lacks a language");
    }
}

#[test]
fn should_find_a_shared_key_for_any_tool_and_none_for_an_unknown_one() {
    assert_eq!(
        describir("", "detalle", Idioma::En),
        comunes::TABLA
            .iter()
            .find(|e| e.clave == "detalle")
            .map(|e| e.texto.en)
    );
    assert_eq!(describir("", "no_existe", Idioma::En), None);
}

fn describe(props: &[(&String, &Value)], clave: &str) -> bool {
    props
        .iter()
        .filter(|(c, _)| c.as_str() == clave)
        .all(|(_, v)| v.get("description").is_some())
}

#[test]
fn should_describe_detalle_and_the_workspace_guard_in_every_tool() {
    for esquema in esquemas(Idioma::Es, false) {
        let props = propiedades(&esquema);
        for clave in ["detalle", "permitir_fuera_del_workspace"] {
            assert!(
                describe(&props, clave),
                "{}.{clave} has no description",
                nombre(&esquema)
            );
        }
    }
}

#[test]
fn should_keep_each_description_short() {
    for esquema in esquemas(Idioma::En, true) {
        let largas: Vec<&str> = tabla(nombre(&esquema))
            .iter()
            .chain(comunes::TABLA)
            .filter(|e| {
                e.texto.es.chars().count() > LARGO_MAXIMO
                    || e.texto.en.chars().count() > LARGO_MAXIMO
            })
            .map(|e| e.clave)
            .collect();
        assert!(
            largas.is_empty(),
            "{}: {largas:?} exceed {LARGO_MAXIMO} characters",
            nombre(&esquema)
        );
    }
}
