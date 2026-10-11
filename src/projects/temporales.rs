//! `proyecto accion:limpiar_temporales`: moves scratch artifacts created by tests
//! and verification runs (`mcp_tmp_*` projects, variants, results and sites) to the
//! workspace recycle bin. Nothing is deleted outright.

use crate::error::Result;
use std::fs;
use std::path::Path;

use serde_json::{json, Value};

use crate::args;
use crate::config::Paths;
use crate::workspace;

/// Prefix shared by every scratch artifact name.
const PREFIJO: &str = "mcp_tmp_";

/// Workspace folders that can hold scratch artifacts (every folder a tool of this
/// server writes into).
const CARPETAS: &[&str] = &[
    "Projects",
    "Results",
    "Sites",
    "Meteo",
    "UserData",
    "UserBatch",
    "ComposPV/Inverters",
    "ComposPV/PVModules",
];

/// True when the file stem starts with the scratch prefix (case-insensitive).
fn es_temporal(nombre: &str) -> bool {
    let stem = nombre.split('.').next().unwrap_or(nombre);
    stem.get(..PREFIJO.len())
        .is_some_and(|p| p.eq_ignore_ascii_case(PREFIJO))
}

/// Scratch files found in one workspace folder, sorted by name.
fn temporales_en(dir: &Path) -> Vec<std::path::PathBuf> {
    let Ok(entradas) = fs::read_dir(dir) else {
        return Vec::new();
    };
    let mut out: Vec<_> = entradas
        .flatten()
        .map(|e| e.path())
        .filter(|p| {
            p.is_file()
                && es_temporal(
                    &p.file_name()
                        .map_or_else(String::new, |n| n.to_string_lossy().into_owned()),
                )
        })
        .collect();
    out.sort();
    out
}

/// `proyecto accion:limpiar_temporales {}`.
pub(crate) fn limpiar_temporales(paths: &Paths, args: &Value) -> Result<Value> {
    let ws = args::workspace_of(paths, args);
    let mut movidos = Vec::new();
    let mut fallidos = Vec::new();
    let mut vistos = std::collections::HashSet::new();
    for carpeta in CARPETAS {
        for origen in temporales_en(&ws.join(carpeta)) {
            if !vistos.insert(origen.clone()) {
                continue;
            }
            let nombre = origen
                .file_name()
                .map_or_else(String::new, |n| n.to_string_lossy().into_owned());
            match workspace::mover_a_papelera(&ws, &origen) {
                Ok(_) => movidos.push(format!("{carpeta}/{nombre}")),
                Err(e) => {
                    fallidos.push(json!({"archivo": format!("{carpeta}/{nombre}"), "error": e}))
                }
            }
        }
    }
    Ok(json!({
        "ok": fallidos.is_empty(),
        "movidos": movidos,
        "fallidos": fallidos,
    }))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn solo_reconoce_el_prefijo_temporal() {
        assert!(es_temporal("mcp_tmp_demo.VC0"));
        assert!(es_temporal("MCP_TMP_demo.PRJ"));
        assert!(!es_temporal("MCP_E2E.VC0"));
        assert!(!es_temporal("mcp_tm.PRJ"));
        assert!(!es_temporal("Ciudad_é.SIT"));
    }

    #[test]
    fn should_recognize_scratch_file_when_prefix_matches() {
        assert!(es_temporal("mcp_tmp_test.VC0"));
        assert!(!es_temporal("production.PRJ"));
    }
}
