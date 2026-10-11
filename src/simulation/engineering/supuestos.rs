//! User-supplied engineering assumptions (rates, lifetimes, costs) with
//! business bounds, so no calculation runs on non-finite or absurd inputs.

use crate::error::{Error, Result};
use serde_json::Value;

/// Admissible interval of one assumption, inclusive.
pub(super) struct Rango {
    pub(super) clave: &'static str,
    pub(super) min: f64,
    pub(super) max: f64,
}

/// Longest economic or carbon horizon accepted, in years.
pub(super) const MAX_ANIOS: u32 = 100;

/// Annual degradation, as a fraction.
pub(super) const DEGRADACION: Rango = Rango {
    clave: "degradacion",
    min: 0.0,
    max: 0.5,
};

/// `rango.clave` from `args` (or `defecto` when absent), finite and inside the range.
pub(super) fn acotado(args: &Value, rango: &Rango, defecto: Option<f64>) -> Result<f64> {
    let valor = match args.get(rango.clave).filter(|v| !v.is_null()) {
        Some(v) => v
            .as_f64()
            .ok_or_else(|| Error::Invalid(format!("'{}' debe ser un número", rango.clave)))?,
        None => defecto.ok_or_else(|| Error::Invalid(format!("Falta '{}'", rango.clave)))?,
    };
    if !valor.is_finite() || valor < rango.min || valor > rango.max {
        return Err(Error::Invalid(format!(
            "'{}'={valor} fuera de rango [{}, {}]",
            rango.clave, rango.min, rango.max
        )));
    }
    Ok(valor)
}

/// Whole number of years `clave` (or `defecto`), in `1..=MAX_ANIOS`.
pub(super) fn anios(args: &Value, clave: &'static str, defecto: f64) -> Result<u32> {
    let rango = Rango {
        clave,
        min: 1.0,
        max: f64::from(MAX_ANIOS),
    };
    let valor = acotado(args, &rango, Some(defecto))?;
    if valor.fract() != 0.0 {
        return Err(Error::Invalid(format!(
            "'{clave}'={valor} debe ser un número entero de años"
        )));
    }
    // In 1..=MAX_ANIOS and integral, so the conversion is exact.
    Ok(valor as u32)
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn should_reject_non_finite_out_of_range_and_fractional_assumptions() {
        assert!(acotado(&json!({"degradacion": 1.5}), &DEGRADACION, None).is_err());
        assert!(acotado(&json!({"degradacion": -0.1}), &DEGRADACION, None).is_err());
        assert!(acotado(&json!({}), &DEGRADACION, None).is_err());
        assert_eq!(
            acotado(&json!({}), &DEGRADACION, Some(0.01)).ok(),
            Some(0.01)
        );
        assert!(anios(&json!({"anios": 2.5}), "anios", 25.0).is_err());
        assert!(anios(&json!({"anios": 1e9}), "anios", 25.0).is_err());
        assert_eq!(anios(&json!({}), "anios", 25.0).ok(), Some(25));
    }
}
