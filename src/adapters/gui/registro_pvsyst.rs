//! The message boxes PVsyst shows are not readable through Win32 (VCL draws
//! their text), but PVsyst writes every one of them to its session log. This
//! reads what PVsyst logged after a given point.

use crate::config::local_app_data;
use serde_json::{json, Value};
use std::fs::File;
use std::io::{BufRead, BufReader, Seek, SeekFrom};
use std::path::PathBuf;

/// Log method of the message boxes shown to the user.
const METODO_MENSAJE: &str = "ShowMessageBox";
const NIVELES_AVISO: [&str; 2] = ["Warn", "Err"];
/// Log noise of an unlicensed run, not about the user's project.
const RUIDO: [&str; 2] = ["InitLicense", "license file"];

/// A point in the current PVsyst log; later lines are the ones of interest.
#[derive(Debug, Clone)]
pub(crate) struct Marca {
    ruta: PathBuf,
    desde: u64,
}

/// `Admin` folders of every PVsyst version installed for this user.
fn carpetas_admin() -> Vec<PathBuf> {
    let Ok(base) = local_app_data() else {
        return Vec::new();
    };
    std::fs::read_dir(base.join("PVsyst"))
        .into_iter()
        .flatten()
        .filter_map(std::result::Result::ok)
        .map(|e| e.path().join("Admin"))
        .filter(|p| p.is_dir())
        .collect()
}

fn log_mas_reciente() -> Option<PathBuf> {
    carpetas_admin()
        .into_iter()
        .filter_map(|dir| std::fs::read_dir(dir).ok())
        .flatten()
        .filter_map(std::result::Result::ok)
        .filter(|e| {
            e.path()
                .extension()
                .is_some_and(|x| x.eq_ignore_ascii_case("log"))
        })
        .max_by_key(|e| e.metadata().and_then(|m| m.modified()).ok())
        .map(|e| e.path())
}

/// Marks the end of PVsyst's current log. `None` when there is no log.
pub(crate) fn marcar() -> Option<Marca> {
    let ruta = log_mas_reciente()?;
    let desde = std::fs::metadata(&ruta).ok()?.len();
    Some(Marca { ruta, desde })
}

/// Last `limite` message-box and warning entries of PVsyst's current log.
pub(crate) fn ultimos_mensajes(limite: usize) -> Value {
    let Some(ruta) = log_mas_reciente() else {
        return json!({ "ruta": null, "count": 0, "mensajes": [] });
    };
    let marca = Marca {
        ruta: ruta.clone(),
        desde: 0,
    };
    let mut mensajes = mensajes_desde(&marca);
    let sobrantes = mensajes.len().saturating_sub(limite);
    mensajes.drain(..sobrantes);
    json!({
        "ruta": ruta.to_string_lossy(),
        "count": mensajes.len(),
        "mensajes": mensajes,
    })
}

/// Columns of a log line: `time|level|class|method|message`.
fn columnas(linea: &str) -> Vec<&str> {
    linea.splitn(5, '|').map(str::trim).collect()
}

fn es_continuacion(c: &[&str]) -> bool {
    c.len() == 5 && c.first() == Some(&"") && c.get(1) == Some(&"")
}

fn anadir_continuacion(salida: &mut [String], cuerpo: &str) {
    if let Some(ultimo) = salida.last_mut() {
        ultimo.push(' ');
        ultimo.push_str(cuerpo);
    }
}

/// Messages PVsyst logged after `marca`: every message box (with its
/// continuation lines) and every warning or error, licence noise excluded.
pub(crate) fn mensajes_desde(marca: &Marca) -> Vec<String> {
    let Ok(mut f) = File::open(&marca.ruta) else {
        return Vec::new();
    };
    if f.seek(SeekFrom::Start(marca.desde)).is_err() {
        return Vec::new();
    }
    let mut salida: Vec<String> = Vec::new();
    let mut mensaje_abierto = false;
    for linea in BufReader::new(f).lines().map_while(std::result::Result::ok) {
        mensaje_abierto = acumular(&mut salida, &linea, mensaje_abierto);
    }
    salida.dedup();
    salida
}

/// Adds `linea` to `salida`; returns whether a message is still open.
fn acumular(salida: &mut Vec<String>, linea: &str, mensaje_abierto: bool) -> bool {
    let c = columnas(linea);
    let cuerpo = c.get(4).copied().unwrap_or("");
    if es_continuacion(&c) {
        if mensaje_abierto && !cuerpo.is_empty() {
            anadir_continuacion(salida, cuerpo);
        }
        return mensaje_abierto;
    }
    if c.len() < 5 || RUIDO.iter().any(|r| linea.contains(r)) {
        return false;
    }
    let metodo = c.get(2).copied().unwrap_or("");
    let nivel = c.get(1).copied().unwrap_or("");
    let es_mensaje = metodo == METODO_MENSAJE || NIVELES_AVISO.contains(&nivel);
    if es_mensaje && !cuerpo.is_empty() {
        salida.push(cuerpo.to_string());
    }
    es_mensaje
}

#[cfg(test)]
#[path = "../../../tests/unit/adapters/gui/registro_pvsyst.rs"]
mod tests;
