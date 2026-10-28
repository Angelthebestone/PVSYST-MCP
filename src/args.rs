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

/// Argument asking for a run that changes nothing and reports what it would do.
pub(crate) const DRY_RUN: &str = "dry_run";
/// Argument that accepts the trial-mode execution cost.
pub(crate) const CONFIRMAR: &str = "confirmar";
/// Answer key a call sets when it needs `CONFIRMAR` before it spends trial runs.
pub(crate) const REQUIERE_CONFIRMACION: &str = "requiere_confirmacion";

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

/// Optional finite JSON number; absent or `null` means `None`.
pub(crate) fn opt_f64(args: &Value, key: &str) -> Result<Option<f64>> {
    match args.get(key) {
        None | Some(Value::Null) => Ok(None),
        Some(v) => v
            .as_f64()
            .filter(|n| n.is_finite())
            .map(Some)
            .ok_or_else(|| Error::Invalid(format!("'{key}' debe ser un número finito"))),
    }
}

/// Optional JSON integer that fits `u32`; absent or `null` means `None`.
pub(crate) fn opt_u32(args: &Value, key: &str) -> Result<Option<u32>> {
    match args.get(key) {
        None | Some(Value::Null) => Ok(None),
        Some(v) => v
            .as_u64()
            .and_then(|n| u32::try_from(n).ok())
            .map(Some)
            .ok_or_else(|| Error::Invalid(format!("'{key}' debe ser un entero positivo"))),
    }
}

/// Optional JSON integer that fits `i32`; absent or `null` means `None`.
pub(crate) fn opt_i32(args: &Value, key: &str) -> Result<Option<i32>> {
    match args.get(key) {
        None | Some(Value::Null) => Ok(None),
        Some(v) => v
            .as_i64()
            .and_then(|n| i32::try_from(n).ok())
            .map(Some)
            .ok_or_else(|| Error::Invalid(format!("'{key}' debe ser un entero"))),
    }
}

/// Required string argument; the error names `keys[0]` and, if given, `hint`.
pub(crate) fn req_str<'a>(args: &'a Value, keys: &[&str], hint: &str) -> Result<&'a str> {
    opt_str(args, keys).ok_or_else(|| missing(keys[0], hint))
}

/// Required string argument where an empty string is a valid value (e.g. file
/// content); only an absent, null or non-string argument counts as missing.
pub(crate) fn req_texto<'a>(args: &'a Value, keys: &[&str], hint: &str) -> Result<&'a str> {
    keys.iter()
        .find_map(|k| args.get(*k)?.as_str())
        .ok_or_else(|| missing(keys[0], hint))
}

/// Required finite number given as a JSON number or a numeric string. Absent or
/// null is `missing`; present but not numeric is an `Invalid` that names the value.
pub(crate) fn req_numero(args: &Value, key: &str, hint: &str) -> Result<f64> {
    let valor = match args.get(key) {
        None | Some(Value::Null) => return Err(missing(key, hint)),
        Some(v) => v,
    };
    let numero = valor
        .as_f64()
        .or_else(|| valor.as_str()?.trim().parse().ok())
        .filter(|n| n.is_finite());
    numero.ok_or_else(|| {
        let recibido = valor
            .as_str()
            .map_or_else(|| valor.to_string(), String::from);
        Error::Invalid(format!(
            "'{key}' debe ser un número finito (recibido '{recibido}')"
        ))
    })
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
#[path = "../tests/unit/args.rs"]
mod tests;
