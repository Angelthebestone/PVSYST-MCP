//! The bundle manifest is a copy of the tool registry: it is what a client
//! reads before `initialize`, so a stale copy is a wrong description.

use super::*;
use crate::idioma::Idioma;
use serde_json::Value;

/// Compiled in, so the test cannot pass by not finding the file.
const MANIFIESTO: &str = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/mcpb/manifest.json"));

/// `(name, description)` of every tool, in registry order, as a client sees it
/// by default: English and without the approximation-gated actions.
fn registro() -> Vec<(String, String)> {
    let mut paths = Paths::para_test(std::env::temp_dir());
    paths.idioma = Idioma::En;
    list(&paths)
        .iter()
        .map(|esquema| {
            let texto = |clave: &str| {
                esquema
                    .get(clave)
                    .and_then(Value::as_str)
                    .unwrap_or_default()
                    .to_string()
            };
            (texto("name"), texto("description"))
        })
        .collect()
}

fn del_manifiesto(manifiesto: &Value) -> Vec<(String, String)> {
    manifiesto
        .get("tools")
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
        .map(|herramienta| {
            let texto = |clave: &str| {
                herramienta
                    .get(clave)
                    .and_then(Value::as_str)
                    .unwrap_or_default()
                    .to_string()
            };
            (texto("name"), texto("description"))
        })
        .collect()
}

#[test]
fn should_match_the_tool_registry_when_the_bundle_manifest_is_read() {
    let manifiesto: Value = match serde_json::from_str(MANIFIESTO) {
        Ok(valor) => valor,
        Err(error) => panic!("mcpb/manifest.json is not JSON: {error}"),
    };

    assert_eq!(
        del_manifiesto(&manifiesto),
        registro(),
        "mcpb/manifest.json does not match the registry: run tools/generar_contrato.py"
    );
}
