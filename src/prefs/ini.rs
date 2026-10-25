//! GUI preferences: reading and editing `%LOCALAPPDATA%\PVsyst\8.1\Admin\PVsyst.ini`.
//! Secrets are masked on read, edits are refused while PVsyst runs, and writes
//! use snapshot + atomic write.

use crate::adapters::gui::gui_running;
use crate::args::{opt_str, req_str};
use crate::config::Paths;
use crate::error::{Error, Result};
use crate::history::{escribir_atomico, raiz_usuario, snapshot_usuario};
use serde_json::{json, Map, Value};
use std::fs;
use std::path::{Path, PathBuf};

pub(crate) const BOM_UTF8: &[u8] = &[0xEF, 0xBB, 0xBF];
pub(crate) const BOM_UTF16LE: &[u8] = &[0xFF, 0xFE];

/// Text encoding of an ini file, kept so edits write it back unchanged.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Codificacion {
    Utf16Le,
    Utf8Bom,
    Utf8,
}

pub(crate) fn ini_path() -> Result<PathBuf> {
    Ok(crate::config::local_app_data()?
        .join("PVsyst")
        .join("8.1")
        .join("Admin")
        .join("PVsyst.ini"))
}

pub(crate) fn leer(ruta: &Path) -> Result<Vec<u8>> {
    fs::read(ruta).map_err(|e| Error::io(format!("No se pudo leer {}", ruta.display()), e))
}

/// Detect the encoding from the BOM; anything without one is UTF-8.
pub(crate) fn decodificar(bytes: &[u8]) -> (String, Codificacion) {
    if let Some(resto) = bytes.strip_prefix(BOM_UTF16LE) {
        let (pares, _) = resto.as_chunks::<2>();
        let unidades: Vec<u16> = pares.iter().map(|c| u16::from_le_bytes(*c)).collect();
        return (String::from_utf16_lossy(&unidades), Codificacion::Utf16Le);
    }
    if let Some(resto) = bytes.strip_prefix(BOM_UTF8) {
        return (
            String::from_utf8_lossy(resto).into_owned(),
            Codificacion::Utf8Bom,
        );
    }
    (
        String::from_utf8_lossy(bytes).into_owned(),
        Codificacion::Utf8,
    )
}

pub(crate) fn codificar(texto: &str, cod: Codificacion) -> Vec<u8> {
    match cod {
        Codificacion::Utf16Le => {
            let mut out = BOM_UTF16LE.to_vec();
            for unidad in texto.encode_utf16() {
                out.extend_from_slice(&unidad.to_le_bytes());
            }
            out
        }
        Codificacion::Utf8Bom => {
            let mut out = BOM_UTF8.to_vec();
            out.extend_from_slice(texto.as_bytes());
            out
        }
        Codificacion::Utf8 => texto.as_bytes().to_vec(),
    }
}

pub(crate) fn seccion_protegida(seccion: &str) -> bool {
    seccion.eq_ignore_ascii_case("API") || seccion.eq_ignore_ascii_case("Web")
}

/// Values in `[API]`/`[Web]` and keys that name secrets are never shown.
pub(crate) fn oculto(seccion: &str, clave: &str) -> bool {
    let c = clave.to_ascii_lowercase();
    seccion_protegida(seccion)
        || ["key", "password", "token", "secret"]
            .iter()
            .any(|p| c.contains(p))
}

/// Parse `[Section]` / `key=value` lines into `{seccion:{clave:valor}}`.
pub(crate) fn parsear(texto: &str) -> Map<String, Value> {
    let mut secciones = Map::new();
    let mut actual: Option<String> = None;
    for linea in texto.lines() {
        let t = linea.trim();
        if t.is_empty() || t.starts_with(';') {
            continue;
        }
        if let Some(nombre) = t.strip_prefix('[').and_then(|r| r.strip_suffix(']')) {
            let nombre = nombre.trim().to_string();
            secciones
                .entry(nombre.clone())
                .or_insert_with(|| Value::Object(Map::new()));
            actual = Some(nombre);
            continue;
        }
        let (Some(sec), Some((clave, valor))) = (actual.as_ref(), t.split_once('=')) else {
            continue;
        };
        let clave = clave.trim();
        let mostrado = if oculto(sec, clave) {
            "***".to_string()
        } else {
            valor.trim().to_string()
        };
        if let Some(Value::Object(m)) = secciones.get_mut(sec) {
            m.insert(clave.to_string(), json!(mostrado));
        }
    }
    secciones
}

/// Replace the value of `clave` inside `[seccion]`, keeping line endings.
pub(crate) fn reemplazar_clave(
    texto: &str,
    seccion: &str,
    clave: &str,
    valor: &str,
) -> Result<String> {
    let mut en_seccion = false;
    let mut hecho = false;
    let mut salida = String::with_capacity(texto.len());
    for pieza in texto.split_inclusive('\n') {
        let contenido = pieza.trim_end_matches(['\r', '\n']);
        let fin = &pieza[contenido.len()..];
        let t = contenido.trim();
        if let Some(nombre) = t.strip_prefix('[').and_then(|r| r.strip_suffix(']')) {
            en_seccion = nombre.trim().eq_ignore_ascii_case(seccion);
        } else if en_seccion && !hecho {
            if let Some((k, _)) = t.split_once('=') {
                if k.trim().eq_ignore_ascii_case(clave) {
                    let sangria = &contenido[..contenido.len() - contenido.trim_start().len()];
                    salida.push_str(&format!("{sangria}{}={valor}{fin}", k.trim()));
                    hecho = true;
                    continue;
                }
            }
        }
        salida.push_str(pieza);
    }
    if hecho {
        Ok(salida)
    } else {
        Err(Error::NotFound(format!(
            "Clave '{clave}' no existe en [{seccion}]"
        )))
    }
}

/// Show the GUI preferences, masking secrets.
///
/// Without `seccion` returns only the key count per section, so routine calls
/// stay small; pass `seccion` to see its keys and values.
/// Pass `clave` together with `seccion` to return only that key.
pub(crate) fn preferencias(_paths: &Paths, args: &Value) -> Result<Value> {
    let ruta = ini_path()?;
    let (texto, _) = decodificar(&leer(&ruta)?);
    let mut secciones = parsear(&texto);
    let clave_pedida = opt_str(args, &["clave"]);
    if clave_pedida.is_some() && opt_str(args, &["seccion"]).is_none() {
        return Err(Error::Invalid(
            "'clave' requiere 'seccion': PVsyst.ini se consulta por sección".into(),
        ));
    }
    if let Some(sel) = opt_str(args, &["seccion"]) {
        let clave = secciones
            .keys()
            .find(|k| k.eq_ignore_ascii_case(sel))
            .cloned()
            .ok_or_else(|| Error::NotFound(format!("Sección '{sel}' no existe en PVsyst.ini")))?;
        let mut valor = secciones.remove(&clave).unwrap_or(Value::Null);
        if let Some(pedida) = clave_pedida {
            valor = filtrar_clave(&valor, pedida)?;
        }
        secciones = Map::from_iter([(clave, valor)]);
        return Ok(json!({
            "archivo": ruta.display().to_string(),
            "secciones": secciones,
        }));
    }
    let conteo: Map<String, Value> = secciones
        .into_iter()
        .map(|(k, v)| (k, Value::from(v.as_object().map_or(0, Map::len))))
        .collect();
    Ok(json!({
        "archivo": ruta.display().to_string(),
        "secciones": conteo,
    }))
}

/// Keeps only `clave` (case-insensitive) from a section's key/value object.
fn filtrar_clave(valor: &Value, clave: &str) -> Result<Value> {
    valor
        .as_object()
        .and_then(|o| o.iter().find(|(k, _)| k.eq_ignore_ascii_case(clave)))
        .map(|(k, v)| Value::Object(Map::from_iter([(k.clone(), v.clone())])))
        .ok_or_else(|| Error::NotFound(format!("Clave '{clave}' no existe en la sección")))
}

/// Set one key in the GUI preferences. The previous file is snapshotted under
/// the per-user MCP state directory (never the workspace), then written atomically.
pub(crate) fn preferencias_editar(args: &Value) -> Result<Value> {
    let seccion = req_str(args, &["seccion"], "seccion")?;
    let clave = req_str(args, &["clave"], "clave")?;
    let valor = req_str(args, &["valor"], "valor")?;
    if gui_running() {
        return Err(Error::Invalid(
            "Cierra PVsyst antes: reescribe PVsyst.ini al salir".into(),
        ));
    }
    if seccion_protegida(seccion) {
        return Err(Error::Invalid(format!(
            "La sección [{seccion}] contiene credenciales y no se edita desde MCP"
        )));
    }
    if valor.contains(['\r', '\n']) {
        return Err(Error::Invalid(format!(
            "Valor con salto de línea en '{clave}'"
        )));
    }
    let ruta = ini_path()?;
    let (texto, cod) = decodificar(&leer(&ruta)?);
    let nuevo = reemplazar_clave(&texto, seccion, clave, valor)?;
    let historial_id = snapshot_usuario(std::slice::from_ref(&ruta), "preferencias_editar")?;
    let historial_dir = raiz_usuario()?.join(&historial_id);
    escribir_atomico(&ruta, &codificar(&nuevo, cod))?;
    Ok(json!({
        "ok": true,
        "archivo": ruta.display().to_string(),
        "seccion": seccion,
        "clave": clave,
        "historial_id": historial_id,
        "historial_dir": historial_dir.display().to_string(),
    }))
}

#[cfg(test)]
#[path = "../../tests/unit/prefs/ini.rs"]
mod tests;
