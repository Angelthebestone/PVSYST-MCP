//! Extracción y consolidación de avisos emitidos por PVsystCLI en sus logs Admin.
//!
//! Lee los logs generados durante la ejecución de simulaciones y extrae
//! los mensajes de error/aviso sin duplicados ni ruido de licencias.

use std::fs::File;
use std::io::BufRead;
use std::path::{Path, PathBuf};
use std::time::SystemTime;

/// Per-hour detail lines; PVsyst already summarises them in the error block
/// ("1879 occurrences of MPP voltage lower than ...").
const MARCADOR_COUNT: &str = "LErrorList.Count =";
const MARCADOR_DETALLE_HORARIO: &str = "Incident global =";
const MARCADOR_ERRORES: &str = "Errors during the simulation:";
const MARGEN_SEGUNDOS: u64 = 2;

#[derive(Default)]
struct AcumuladorAvisos {
    resumen_errores: Vec<String>,
    otros_avisos: Vec<String>,
    en_bloque_errores: bool,
}

fn extraer_mensaje(linea: &str) -> &str {
    let partes: Vec<&str> = linea.splitn(5, '|').collect();
    if partes.len() == 5 {
        partes[4].trim()
    } else {
        partes.last().map(|s| s.trim()).unwrap_or("")
    }
}

fn es_continuacion(linea: &str) -> bool {
    let mut iter = linea.split('|');
    let primero = iter.next().unwrap_or("");
    if primero.starts_with("...") {
        return true;
    }
    if !primero.is_empty() && primero.chars().all(|c| c.is_whitespace()) {
        return true;
    }
    let segundo = iter.next().unwrap_or("");
    segundo.trim().is_empty() && !linea.trim().is_empty()
}

fn es_ruido_licencia(linea: &str) -> bool {
    let minus = linea.to_ascii_lowercase();
    minus.contains("license file")
        || minus.contains("no license file")
        || minus.contains("license.lic")
        || (minus.contains("tenvmanager") && minus.contains("initlicense"))
}

fn nivel(linea: &str) -> Option<&str> {
    let mut iter = linea.splitn(3, '|');
    let _col0 = iter.next()?;
    let col1 = iter.next()?;
    Some(col1.trim())
}

fn es_aviso_o_error(linea: &str) -> bool {
    let Some(niv) = nivel(linea) else {
        return false;
    };
    niv.eq_ignore_ascii_case("warn")
        || niv.eq_ignore_ascii_case("warning")
        || niv.eq_ignore_ascii_case("err")
        || niv.eq_ignore_ascii_case("error")
}

fn procesar_linea(acum: &mut AcumuladorAvisos, linea: &str) {
    if linea.trim().is_empty() || linea.contains(MARCADOR_COUNT) {
        return;
    }
    if acum.en_bloque_errores {
        if es_continuacion(linea) {
            let msg = extraer_mensaje(linea);
            if !msg.is_empty() && !msg.starts_with(MARCADOR_DETALLE_HORARIO) {
                acum.resumen_errores.push(msg.to_string());
            }
            return;
        }
        acum.en_bloque_errores = false;
    }
    if extraer_mensaje(linea).starts_with(MARCADOR_ERRORES) {
        acum.en_bloque_errores = true;
        return;
    }
    if es_ruido_licencia(linea) || !es_aviso_o_error(linea) {
        return;
    }
    let msg = extraer_mensaje(linea);
    if !msg.is_empty() && !acum.otros_avisos.iter().any(|m| m == msg) {
        acum.otros_avisos.push(msg.to_string());
    }
}

fn ensamblar_avisos(acum: AcumuladorAvisos) -> Vec<String> {
    let mut out = acum.resumen_errores;
    for aviso in acum.otros_avisos {
        if !out.contains(&aviso) {
            out.push(aviso);
        }
    }
    out
}

/// Parsea un stream de log línea a línea sin alocaciones intermedias en bucle.
pub(crate) fn parsear_log<R: BufRead>(mut reader: R) -> Vec<String> {
    let mut acum = AcumuladorAvisos::default();
    let mut linea = String::new();

    while reader.read_line(&mut linea).unwrap_or(0) > 0 {
        let limpia = linea.trim_end_matches(['\r', '\n']);
        procesar_linea(&mut acum, limpia);
        linea.clear();
    }

    ensamblar_avisos(acum)
}

fn archivos_recientes(dir: &Path, inicio: SystemTime) -> Vec<PathBuf> {
    let Ok(entries) = std::fs::read_dir(dir) else {
        return Vec::new();
    };
    let umbral = inicio
        .checked_sub(std::time::Duration::from_secs(MARGEN_SEGUNDOS))
        .unwrap_or(inicio);
    let mut lista: Vec<(PathBuf, SystemTime)> = Vec::new();
    for entry in entries.flatten() {
        let p = entry.path();
        let es_log = p
            .file_name()
            .and_then(|n| n.to_str())
            .is_some_and(|n| n.ends_with(".log"));
        if !es_log {
            continue;
        }
        let Ok(meta) = entry.metadata() else {
            continue;
        };
        let ctime = meta.created().unwrap_or(std::time::UNIX_EPOCH);
        let mtime = meta.modified().unwrap_or(std::time::UNIX_EPOCH);
        let rec = ctime.max(mtime);
        if rec >= umbral {
            lista.push((p, rec));
        }
    }
    lista.sort_by(|a, b| {
        b.1.cmp(&a.1)
            .then_with(|| b.0.file_name().cmp(&a.0.file_name()))
    });
    lista.into_iter().map(|(p, _)| p).collect()
}

/// Extrae los avisos consolidados de los logs creados a partir de `inicio`.
pub(crate) fn avisos_simulacion(inicio: SystemTime) -> Vec<String> {
    let Ok(dir) =
        crate::config::local_app_data().map(|b| b.join("PVsystCLI").join("8.1").join("Admin"))
    else {
        return Vec::new();
    };
    if !dir.exists() {
        return Vec::new();
    }
    let archivos = archivos_recientes(&dir, inicio);
    let mut todos = Vec::new();
    for ruta in archivos {
        let Ok(file) = File::open(&ruta) else {
            continue;
        };
        let avisos = parsear_log(std::io::BufReader::new(file));
        for a in avisos {
            if !todos.contains(&a) {
                todos.push(a);
            }
        }
    }
    todos
}

#[cfg(test)]
#[path = "../../tests/unit/simulation/avisos_pvsyst.rs"]
mod tests;
