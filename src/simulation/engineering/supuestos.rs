//! User-supplied engineering assumptions with business bounds.

use crate::args::missing;
use crate::error::{Error, Result};
use serde_json::Value;

/// Admissible interval of one assumption, inclusive.
pub(super) struct Rango {
    pub(super) clave: &'static str,
    pub(super) min: f64,
    pub(super) max: f64,
}

/// `rango.clave` from `args` (or `defecto` when absent), finite and inside the range.
pub(super) fn acotado(args: &Value, rango: &Rango, defecto: Option<f64>) -> Result<f64> {
    let valor = match args.get(rango.clave).filter(|v| !v.is_null()) {
        Some(v) => v
            .as_f64()
            .ok_or_else(|| Error::Invalid(format!("'{}' debe ser un número", rango.clave)))?,
        None => defecto.ok_or_else(|| missing(rango.clave, "número finito"))?,
    };
    if !valor.is_finite() || valor < rango.min || valor > rango.max {
        return Err(Error::Invalid(format!(
            "'{}'={valor} fuera de rango [{}, {}]",
            rango.clave, rango.min, rango.max
        )));
    }
    Ok(valor)
}

#[cfg(test)]
#[path = "../../../tests/unit/simulation/engineering/supuestos.rs"]
mod tests;
