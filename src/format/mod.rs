//! Core PVsyst file formats and codecs (pure: no I/O, no MCP, no CLI; uses crate::error for typed error reporting).
//! Every other module may depend on this one.

pub(crate) mod accion;
pub(crate) mod campos_numericos;
pub(crate) mod cierre;
pub(crate) mod delete;
pub(crate) mod edit;
pub(crate) mod insert;
mod lineas;
pub(crate) mod object;
pub(crate) mod real48;
pub(crate) mod seccion;
pub(crate) mod selector;
pub(crate) mod span;

/// Standard CRLF line terminator as string slice.
pub(crate) const CRLF: &str = "\r\n";

/// Decodes bytes as Latin-1 (ISO-8859-1) string.
pub(crate) fn decodificar_latin1(b: &[u8]) -> String {
    b.iter().map(|&c| c as char).collect()
}

/// A number written by PVsyst or a user: trimmed, decimal comma accepted. `NaN` and
/// infinities are refused, so no reading or comparison is built on them.
pub(crate) fn numero_texto(texto: &str) -> Option<f64> {
    texto
        .trim()
        .replace(',', ".")
        .parse::<f64>()
        .ok()
        .filter(|n| n.is_finite())
}

/// [`numero_texto`] for a JSON value: a number, or a string holding one.
pub(crate) fn numero_json(valor: &Value) -> Option<f64> {
    match valor {
        Value::Number(n) => n.as_f64(),
        Value::String(t) => numero_texto(t),
        _ => None,
    }
}

/// Encodes text as Latin-1; `None` when a character is outside it (above U+00FF).
pub(crate) fn codificar_latin1(texto: &str) -> Option<Vec<u8>> {
    texto
        .chars()
        .map(|c| u8::try_from(u32::from(c)).ok())
        .collect()
}

/// Decodes bytes as UTF-8 if valid, falling back to Latin-1.
pub(crate) fn decodificar_texto_flexible(b: &[u8]) -> String {
    match std::str::from_utf8(b) {
        Ok(s) => s.to_string(),
        Err(_) => decodificar_latin1(b),
    }
}

use serde_json::Value;

/// Whether `nuevo` is the value the field already holds, so its line stays untouched.
pub(crate) fn sin_cambio(actual: &str, nuevo: &str) -> bool {
    actual.trim() == nuevo.trim()
}

/// Render a JSON value as PVsyst field text: strings unquoted, the rest as JSON.
pub(crate) fn value_text(v: &Value) -> String {
    match v {
        Value::String(s) => s.clone(),
        other => other.to_string(),
    }
}

/// Round `x` to `decimales` decimal places.
pub(crate) fn redondear(x: f64, decimales: i32) -> f64 {
    let f = 10f64.powi(decimales);
    (x * f).round() / f
}

#[cfg(test)]
#[path = "../../tests/unit/format/mod.rs"]
mod tests;
