//! The electrical figures of a module that sizing and the doctor need, read
//! from either `.PAN` layout returned by [`super::parse_pan`].
//!
//! The two layouts name their parameters differently (`vmpp_v` in the legacy
//! binary record, `vmpp` in the text one); this value object is the single
//! place that knows both, so callers cannot read a key that only one layout has.

use serde_json::Value;

/// Pnom, Voc, Vmpp and optional temperature coefficients of a module.
#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) struct DatosModulo {
    pnom_w: f64,
    voc_v: f64,
    vmpp_v: f64,
    mu_voc_mv_c: Option<f64>,
    mu_pnom_pct_c: Option<f64>,
}

/// First numeric value among `claves` inside `parameters`.
fn parametro(v: &Value, claves: &[&str]) -> Option<f64> {
    claves
        .iter()
        .find_map(|c| v.get("parameters")?.get(c)?.as_f64())
}

impl DatosModulo {
    /// Reads the figures from a parsed module (`parse_pan` / `leer_modulo`
    /// output). `None` when Pnom, Voc or Vmpp is missing.
    pub(crate) fn de_lectura(v: &Value) -> Option<Self> {
        Some(Self {
            pnom_w: v.get("pnom")?.as_f64()?,
            voc_v: v.get("voc")?.as_f64()?,
            vmpp_v: parametro(v, &["vmpp_v", "vmpp"])?,
            mu_voc_mv_c: parametro(v, &["mu_voc_mv_c", "mu_voc"]),
            mu_pnom_pct_c: parametro(v, &["mu_pnom_pct_c", "mu_pnom", "mu_pmpp", "gamma"]),
        })
    }

    /// Nominal power (W).
    pub(crate) fn pnom_w(&self) -> f64 {
        self.pnom_w
    }

    /// Open-circuit voltage at STC (V).
    pub(crate) fn voc_v(&self) -> f64 {
        self.voc_v
    }

    /// Maximum-power-point voltage at STC (V).
    pub(crate) fn vmpp_v(&self) -> f64 {
        self.vmpp_v
    }

    /// Temperature coefficient of Voc (mV/°C), when the file has it.
    pub(crate) fn mu_voc_mv_c(&self) -> Option<f64> {
        self.mu_voc_mv_c
    }

    /// Temperature coefficient of Pnom / Vmpp (%/°C), when the file has it.
    pub(crate) fn mu_pnom_pct_c(&self) -> Option<f64> {
        self.mu_pnom_pct_c
    }
}
