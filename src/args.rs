//! Tool-argument extraction shared by every tool. Each argument has a canonical
//! (Spanish) name followed by accepted aliases; required ones fail fast with a
//! uniform message before any I/O happens.
//!
//! Inputs at the MCP tool boundary are pre-validated against each tool's declared
//! schema (checking required presence and ensuring present properties match
//! their declared JSON types). The functions here extract arguments with alias
//! resolution, defaults and fallbacks for internal domain logic.
use crate::config::Paths;
use crate::error::{Error, Result};
use serde_json::Value;
use std::path::PathBuf;

/// First non-empty string among `keys` (canonical name first, then aliases).
/// Inputs are pre-validated at the MCP boundary against declared schemas.
pub(crate) fn opt_str<'a>(args: &'a Value, keys: &[&str]) -> Option<&'a str> {
    keys.iter()
        .filter_map(|k| args.get(*k)?.as_str())
        .find(|s| !s.is_empty())
}

/// Boolean switch among `keys`; absent or non-boolean means `false`.
/// Inputs are pre-validated at the MCP boundary against declared schemas.
pub(crate) fn flag(args: &Value, keys: &[&str]) -> bool {
    keys.iter()
        .find_map(|k| args.get(*k)?.as_bool())
        .unwrap_or(false)
}

/// Required string argument; the error names `keys[0]` and, if given, `hint`.
pub(crate) fn req_str<'a>(args: &'a Value, keys: &[&str], hint: &str) -> Result<&'a str> {
    opt_str(args, keys).ok_or_else(|| missing(keys[0], hint))
}

/// Uniform "missing argument" error, also used for non-string arguments.
pub(crate) fn missing(key: &str, hint: &str) -> Error {
    Error::MissingArg {
        key: key.to_string(),
        hint: hint.to_string(),
    }
}

/// Workspace for a tool call: the `workspace` argument if given, else the default.
///
/// When supplied, this argument deliberately replaces the default workspace root
/// for the current tool call, defining a new confinement boundary. This enables
/// local multi-workspace workflows; the MCP client is assumed to be trusted.
pub(crate) fn workspace_of(paths: &Paths, args: &Value) -> PathBuf {
    opt_str(args, &["workspace"]).map_or_else(|| paths.workspace.clone(), PathBuf::from)
}

/// Component kind: the `tipo` argument, else the uppercased extension of `archivo`.
pub(crate) fn tipo_de(args: &Value, archivo: &str) -> Option<String> {
    opt_str(args, &["tipo"]).map(str::to_string).or_else(|| {
        std::path::Path::new(archivo)
            .extension()
            .and_then(|e| e.to_str())
            .map(str::to_uppercase)
    })
}

/// Up to 5 likely matches for `objetivo`: case-insensitive substring hits first,
/// then names within Levenshtein distance 3.
pub(crate) fn candidatos(objetivo: &str, opciones: &[String]) -> Vec<String> {
    let needle = objetivo.to_lowercase();
    let mut out: Vec<String> = opciones
        .iter()
        .filter(|o| {
            let low = o.to_lowercase();
            low.contains(&needle) || needle.contains(&low)
        })
        .cloned()
        .collect();
    let mut cercanos: Vec<(usize, String)> = opciones
        .iter()
        .filter(|o| !out.contains(o))
        .map(|o| (distancia(&needle, &o.to_lowercase()), o.clone()))
        .filter(|(d, _)| *d <= 3)
        .collect();
    cercanos.sort();
    out.extend(cercanos.into_iter().map(|(_, o)| o));
    out.truncate(5);
    out
}

/// Uniform "not found" message listing candidates or the folder that was searched.
pub(crate) fn no_encontrado(tipo: &str, pedido: &str, opciones: &[String], carpeta: &str) -> Error {
    let sugeridos = candidatos(pedido, opciones);
    Error::NotFound(if sugeridos.is_empty() {
        format!("{tipo} '{pedido}' no encontrado; no hay ninguno en {carpeta}")
    } else {
        format!(
            "{tipo} '{pedido}' no encontrado; candidatos: {}",
            sugeridos.join(", ")
        )
    })
}

/// Levenshtein edit distance over chars.
fn distancia(a: &str, b: &str) -> usize {
    let b: Vec<char> = b.chars().collect();
    let mut prev: Vec<usize> = (0..=b.len()).collect();
    for (i, ca) in a.chars().enumerate() {
        let mut cur = vec![i + 1];
        for (j, cb) in b.iter().enumerate() {
            let sustitucion = prev[j] + usize::from(ca != *cb);
            cur.push(sustitucion.min(prev[j + 1] + 1).min(cur[j] + 1));
        }
        prev = cur;
    }
    prev[b.len()]
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn aliases_are_tried_in_order_and_empty_counts_as_missing() {
        let args = json!({ "proyecto": "", "project": "P" });
        assert_eq!(opt_str(&args, &["proyecto", "project"]), Some("P"));
        assert_eq!(opt_str(&args, &["nada"]), None);
    }

    #[test]
    fn required_error_names_canonical_key_and_hint() {
        let args = json!({});
        assert_eq!(
            req_str(&args, &["comando", "command"], "e.g. help")
                .unwrap_err()
                .to_string(),
            "Missing required argument 'comando' (e.g. help)"
        );
        assert_eq!(
            req_str(&args, &["archivo"], "").unwrap_err().to_string(),
            "Missing required argument 'archivo'"
        );
    }

    #[test]
    fn kind_comes_from_tipo_or_the_file_extension() {
        assert_eq!(
            tipo_de(&json!({"tipo": "OND"}), "x.PAN").as_deref(),
            Some("OND")
        );
        assert_eq!(
            tipo_de(&json!({"tipo": ""}), "mod.pan").as_deref(),
            Some("PAN")
        );
        assert_eq!(tipo_de(&json!({}), "sin_extension"), None);
    }

    #[test]
    fn candidates_prefer_substrings_then_close_names() {
        let opts: Vec<String> = ["PVModule_Canadian.PAN", "Inv_50kW.OND", "Other.PAN"]
            .map(String::from)
            .to_vec();
        assert_eq!(candidatos("canadian", &opts), vec!["PVModule_Canadian.PAN"]);
        assert_eq!(candidatos("Inv_50kV.OND", &opts), vec!["Inv_50kW.OND"]);
        assert!(candidatos("zzzzzzzzzz", &opts).is_empty());
    }

    #[test]
    fn not_found_message_lists_candidates_or_folder() {
        let opts = vec!["Planta.PRJ".to_string()];
        assert_eq!(
            no_encontrado("Proyecto", "planta", &opts, "C:/ws").to_string(),
            "Proyecto 'planta' no encontrado; candidatos: Planta.PRJ"
        );
        assert_eq!(
            no_encontrado("Proyecto", "xyz", &[], "C:/ws").to_string(),
            "Proyecto 'xyz' no encontrado; no hay ninguno en C:/ws"
        );
    }

    mod props {
        use super::*;
        use proptest::prelude::*;

        proptest! {
            #[test]
            fn candidates_include_exact_match_when_present(
                word in "[a-z]{1,8}",
            ) {
                let options = vec![word.clone(), "other".to_string()];
                let found = candidatos(&word, &options);
                prop_assert!(found.contains(&word));
            }

            #[test]
            fn candidates_are_few_and_come_from_the_options(
                needle in "[a-z]{0,6}",
                options in prop::collection::vec("[a-z]{0,8}", 0..12),
            ) {
                let found = candidatos(&needle, &options);
                prop_assert!(found.len() <= 5);
                prop_assert!(found.iter().all(|c| options.contains(c)));
            }
        }
    }
}
