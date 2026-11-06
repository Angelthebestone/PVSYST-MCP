//! Atomic file writes with automatic history snapshots.

use crate::args::{flag, req_str, req_texto, workspace_of};
use crate::config::Paths;
use crate::error::{Error, Result};
use crate::history;
use crate::idioma::{Idioma, Texto};
use crate::workspace::base64::decode_base64;
use crate::workspace::rutas::{resolver_ruta, Alcance};
use serde_json::{json, Value};
use std::fs;
use std::path::Path;

/// An empty write has nothing to verify, so it is refused unless `base64` is explicit.
const CONTENIDO_VACIO: Texto = Texto {
    es: "'contenido' no puede estar vacío: un archivo de 0 bytes no tiene nada que verificar. Pasa 'base64': true si el contenido vacío es deliberado.",
    en: "'contenido' must not be empty: a 0-byte file has nothing to verify. Pass 'base64': true if the empty content is deliberate.",
};

/// Overwriting is this tool's default; an explicit `sobrescribir: false` vetoes it.
fn exigir_sobrescritura(path: &Path, args: &Value) -> Result<()> {
    let pide = args.get("sobrescribir").and_then(Value::as_bool);
    if pide != Some(false) || !path.exists() {
        return Ok(());
    }
    Err(Error::Invalid(SOBRESCRITURA_RECHAZADA.texto_con(
        Idioma::de_entorno(),
        &[("archivo", &path.to_string_lossy())],
    )))
}

/// Refusal of an overwrite vetoed by the caller.
const SOBRESCRITURA_RECHAZADA: Texto = Texto {
    es: "'{archivo}' ya existe y se pidió 'sobrescribir': false: no se escribió nada",
    en: "'{archivo}' already exists and 'sobrescribir': false was requested: nothing was written",
};

pub(crate) fn escribir_archivo(paths: &Paths, args: &Value) -> Result<Value> {
    let workspace = workspace_of(paths, args);
    let p = req_str(args, &["archivo"], "")?;
    let path = resolver_ruta(&workspace, p, Alcance::de_args(args))?;
    let bytes = extraer_bytes_acotados(args)?;

    validar_formato_pvsyst(&path, &bytes)?;

    exigir_sobrescritura(&path, args)?;
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .map_err(|e| Error::io(format!("Cannot create {}", parent.display()), e))?;
    }
    let historial_id = history::reemplazar(&workspace, &path, &bytes, "escribir_archivo")?;
    Ok(json!({
        "path": path.to_string_lossy(),
        "bytes_written": bytes.len(),
        "historial_id": historial_id,
    }))
}

fn extraer_bytes_acotados(args: &Value) -> Result<Vec<u8>> {
    let content = req_texto(args, &["contenido"], "")?;
    let es_base64 = flag(args, &["base64"]);
    if content.is_empty() && !es_base64 {
        return Err(Error::Invalid(
            CONTENIDO_VACIO.texto_con(Idioma::de_entorno(), &[]),
        ));
    }
    let limite_texto = if es_base64 {
        crate::workspace::MAX_BYTES_MAXIMO.saturating_mul(4) / 3 + 16
    } else {
        crate::workspace::MAX_BYTES_MAXIMO
    };
    if content.len() > limite_texto {
        return Err(Error::Invalid(format!(
            "Contenido a escribir excede el límite máximo permitido ({} bytes, máx {})",
            content.len(),
            limite_texto
        )));
    }
    let bytes = if es_base64 {
        decode_base64(content)?
    } else {
        content.as_bytes().to_vec()
    };
    if bytes.len() > crate::workspace::MAX_BYTES_MAXIMO {
        return Err(Error::Invalid(format!(
            "Bytes a escribir exceden el límite máximo permitido ({} bytes, máx {})",
            bytes.len(),
            crate::workspace::MAX_BYTES_MAXIMO
        )));
    }
    Ok(bytes)
}

fn validar_formato_pvsyst(path: &Path, bytes: &[u8]) -> Result<()> {
    let file_name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");
    // One rule for every written name: ':' (alternate streams), trailing dot or space,
    // control characters and device names (`CON`, `CONIN$`...).
    crate::workspace::validar_nombre(file_name)?;
    let normalizado = file_name;
    if normalizado.eq_ignore_ascii_case("Currency.dat") {
        return Err(Error::Invalid(
            "solo lectura: PVsyst lo lee de su instalación (DataRO) y no está demostrado que lea una copia del workspace".into(),
        ));
    }
    let ext = Path::new(normalizado)
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("")
        .to_ascii_uppercase();
    match ext.as_str() {
        "SHD" | "BLD" | "SHO" => Err(Error::Invalid(
            "escena 3D: solo lectura; el formato de las mallas no está demostrado, créala con la GUI de PVsyst (Construcción / Perspectiva)".into(),
        )),
        "MEF" => crate::meteo_mef::validar_mef(bytes),
        _ => Ok(()),
    }
}

pub(crate) fn comprobar_coincidencia_exacta(
    original: &[u8],
    reserializado: &[u8],
    tipo_formato: &str,
) -> Result<()> {
    if original == reserializado {
        return Ok(());
    }
    let min_len = original.len().min(reserializado.len());
    for i in 0..min_len {
        if original[i] != reserializado[i] {
            return Err(Error::Invalid(format!(
                "{tipo_formato}: discrepancia en byte offset {i}: original 0x{:02X} ('{}'), re-serializado 0x{:02X} ('{}')",
                original[i],
                char_imprimible(original[i]),
                reserializado[i],
                char_imprimible(reserializado[i])
            )));
        }
    }
    Err(Error::Invalid(format!(
        "{tipo_formato}: discrepancia de longitud en offset {min_len}: original {} bytes, re-serializado {} bytes",
        original.len(),
        reserializado.len()
    )))
}

fn char_imprimible(b: u8) -> char {
    if b.is_ascii_graphic() || b == b' ' {
        b as char
    } else {
        '.'
    }
}

#[cfg(test)]
#[path = "../../tests/unit/workspace/escritura.rs"]
mod tests;
