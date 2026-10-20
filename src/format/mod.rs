//! Core PVsyst file formats (pure: no I/O, no MCP, no CLI). Every other
//! module may depend on this one; it depends on nothing in the crate.

pub(crate) mod cierre;
pub(crate) mod delete;
pub(crate) mod edit;
pub(crate) mod insert;
pub(crate) mod object;
pub(crate) mod seccion;
pub(crate) mod selector;
pub(crate) mod span;

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
