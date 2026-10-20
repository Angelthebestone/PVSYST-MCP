//! PVsystCLI admin logs and session diagnostics.

use super::ini::{decodificar, leer};
use crate::args::flag;
use crate::config::Paths;
use crate::error::{Error, Result};
use serde_json::{json, Map, Value};
use std::fs;
use std::path::{Path, PathBuf};

pub(crate) fn cli_admin_dir() -> Result<PathBuf> {
    let base = std::env::var("LOCALAPPDATA")
        .map_err(|_| Error::Invalid("LOCALAPPDATA no está definido".into()))?;
    Ok(PathBuf::from(base)
        .join("PVsystCLI")
        .join("8.1")
        .join("Admin"))
}

/// Files in `dir` whose name satisfies `pred`, newest name first.
pub(crate) fn listar(dir: &Path, pred: impl Fn(&str) -> bool) -> Result<Vec<PathBuf>> {
    if !dir.exists() {
        return Ok(Vec::new());
    }
    let entries = fs::read_dir(dir)
        .map_err(|e| Error::io(format!("No se pudo leer {}", dir.display()), e))?;
    let mut out: Vec<PathBuf> = entries
        .flatten()
        .map(|e| e.path())
        .filter(|p| p.file_name().and_then(|n| n.to_str()).is_some_and(&pred))
        .collect();
    out.sort_by(|a, b| b.file_name().cmp(&a.file_name()));
    Ok(out)
}

/// Log level is the second `|`-separated field (`HH:MM:SS|Level|…`).
pub(crate) fn nivel(linea: &str) -> Option<&str> {
    linea.split('|').nth(1).map(str::trim)
}

/// Max characters per log line kept in responses (the file keeps it whole).
const ANCHO_LINEA: usize = 220;
/// Recent lines listed per log file.
const LINEAS_COLA: usize = 8;
/// Non-Norm (notice/error) lines listed per log file.
const LINEAS_AVISO: usize = 6;

fn recortar_linea(linea: &str) -> String {
    if linea.chars().count() <= ANCHO_LINEA {
        return linea.to_string();
    }
    let corte: String = linea.chars().take(ANCHO_LINEA).collect();
    format!("{corte}…")
}

pub(crate) fn resumir_log(ruta: &Path, texto: &str, solo_errores: bool) -> Value {
    let lineas: Vec<&str> = texto.lines().collect();
    let avisos: Vec<&str> = lineas
        .iter()
        .copied()
        .filter(|l| nivel(l).is_some_and(|n| n != "Norm"))
        .collect();
    let cola_avisos = &avisos[avisos.len().saturating_sub(LINEAS_AVISO)..];
    let mut resumen = json!({
        "archivo": ruta.display().to_string(),
        "lineas_no_norm": cola_avisos
            .iter()
            .map(|l| recortar_linea(l))
            .collect::<Vec<_>>(),
    });
    if avisos.len() > LINEAS_AVISO {
        resumen["lineas_no_norm_total"] = json!(avisos.len());
    }
    if !solo_errores {
        let cola = &lineas[lineas.len().saturating_sub(LINEAS_COLA)..];
        resumen["ultimas"] = json!(cola.iter().map(|l| recortar_linea(l)).collect::<Vec<_>>());
    }
    resumen
}

/// Per-command counts from the newest `cli-session-*.json`, or `null`.
pub(crate) fn ultima_sesion(dir: &Path) -> Result<Value> {
    let candidatos = listar(dir, |n| {
        n.starts_with("cli-session-") && n.ends_with(".json")
    })?;
    let Some(ruta) = candidatos.into_iter().next() else {
        return Ok(Value::Null);
    };
    let (texto, _) = decodificar(&leer(&ruta)?);
    let raiz: Value = serde_json::from_str(&texto)
        .map_err(|e| Error::Format(format!("Sesión CLI ilegible {}: {e}", ruta.display())))?;
    let comandos: Map<String, Value> = match raiz.get("commands").and_then(Value::as_object) {
        Some(m) => m
            .iter()
            .map(|(nombre, v)| {
                (
                    nombre.clone(),
                    json!({
                        "count": v.get("count").cloned().unwrap_or(Value::Null),
                        "nb_crash": v.get("nb_crash").cloned().unwrap_or(Value::Null),
                    }),
                )
            })
            .collect(),
        None => Map::new(),
    };
    Ok(json!({
        "archivo": ruta.display().to_string(),
        "comandos": comandos,
    }))
}

/// Summarise the newest PVsystCLI admin logs and the newest session file.
pub(crate) fn logs_cli(_paths: &Paths, args: &Value) -> Result<Value> {
    let n = args
        .get("n")
        .and_then(Value::as_u64)
        .unwrap_or(1)
        .clamp(1, 20) as usize;
    let solo_errores = flag(args, &["solo_errores"]);
    let dir = cli_admin_dir()?;
    let mut logs = Vec::new();
    let candidatos = listar(&dir, |n| n.ends_with(".log"))?;
    for ruta in candidatos.into_iter().take(n) {
        let (texto, _) = decodificar(&leer(&ruta)?);
        logs.push(resumir_log(&ruta, &texto, solo_errores));
    }
    let sesion = ultima_sesion(&dir)?;
    Ok(json!({
        "directorio": dir.display().to_string(),
        "logs": logs,
        "sesion": sesion,
    }))
}

#[cfg(test)]
#[path = "../../tests/unit/prefs/diagnostico.rs"]
mod tests;
