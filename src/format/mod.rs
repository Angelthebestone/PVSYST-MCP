//! Core PVsyst file formats and codecs (pure: no I/O, no MCP, no CLI; uses crate::error for typed error reporting).
//! Every other module may depend on this one.

pub(crate) mod cierre;
pub(crate) mod delete;
pub(crate) mod edit;
pub(crate) mod insert;
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

/// Decodes bytes as UTF-8 if valid, falling back to Latin-1.
pub(crate) fn decodificar_texto_flexible(b: &[u8]) -> String {
    match std::str::from_utf8(b) {
        Ok(s) => s.to_string(),
        Err(_) => decodificar_latin1(b),
    }
}

use serde_json::Value;

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
