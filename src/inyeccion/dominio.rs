//! Grid-injection limit parameters and their `pvSystem` field changes.

use crate::args::{missing, opt_f64};
use crate::error::{Error, Result};
use serde_json::{json, Value};

const MIN_TAN_PHI: f64 = -1.0;
const MAX_TAN_PHI: f64 = 1.0;
const DECIMALES_POTENCIA: usize = 1;
const DECIMALES_TAN_PHI: usize = 3;

/// Whether the injection limit has its own separate loss.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum PerdidaSeparada {
    Si,
    No,
}

impl PerdidaSeparada {
    pub(crate) fn de_bool(separada: bool) -> Self {
        if separada {
            Self::Si
        } else {
            Self::No
        }
    }

    fn valor(self) -> &'static str {
        match self {
            Self::Si => "True",
            Self::No => "False",
        }
    }
}

/// Raw arguments for the injection limit.
#[derive(Debug, Clone, Copy)]
pub(crate) struct EntradaInyeccion {
    pub(crate) potencia_kw: Option<f64>,
    pub(crate) perdida_separada: Option<bool>,
    pub(crate) tan_phi: Option<f64>,
}

impl EntradaInyeccion {
    pub(crate) fn de_args(args: &Value) -> Result<Self> {
        Ok(Self {
            potencia_kw: opt_f64(args, "potencia_kw")?,
            perdida_separada: args.get("perdida_separada").and_then(Value::as_bool),
            tan_phi: opt_f64(args, "tan_phi")?,
        })
    }
}

/// Validated injection limit.
#[derive(Debug, Clone, PartialEq)]
pub(crate) struct ParametrosInyeccion {
    potencia_kw: f64,
    perdida_separada: PerdidaSeparada,
    tan_phi: Option<f64>,
}

impl ParametrosInyeccion {
    pub(crate) fn nuevo(entrada: EntradaInyeccion) -> Result<Self> {
        let potencia_kw = entrada
            .potencia_kw
            .ok_or_else(|| missing("potencia_kw", "potencia límite en kW"))?;
        if potencia_kw <= 0.0 {
            return Err(Error::Invalid(format!(
                "'potencia_kw' debe ser mayor que 0: {potencia_kw}"
            )));
        }
        if let Some(tan_phi) = entrada.tan_phi {
            if !(MIN_TAN_PHI..=MAX_TAN_PHI).contains(&tan_phi) {
                return Err(Error::Invalid(format!(
                    "'tan_phi' debe estar entre {MIN_TAN_PHI} y {MAX_TAN_PHI}: {tan_phi}"
                )));
            }
        }
        Ok(Self {
            potencia_kw,
            perdida_separada: PerdidaSeparada::de_bool(entrada.perdida_separada.unwrap_or(false)),
            tan_phi: entrada.tan_phi,
        })
    }

    pub(crate) fn como_json(&self) -> Value {
        json!({
            "potencia_kw": self.potencia_kw,
            "perdida_separada": matches!(self.perdida_separada, PerdidaSeparada::Si),
            "tan_phi": self.tan_phi,
        })
    }

    /// `pvSystem` changes for the injection limit.
    pub(crate) fn cambios(&self) -> Vec<(String, Option<String>)> {
        let base = [
            (
                "PLimGrid".to_string(),
                Some(format!("{:.*}", DECIMALES_POTENCIA, self.potencia_kw)),
            ),
            (
                "LimGrSeparLoss".to_string(),
                Some(self.perdida_separada.valor().to_string()),
            ),
        ];
        let tan_phi = self.tan_phi.map(|valor| {
            (
                "TanPhi".to_string(),
                Some(format!("{:.*}", DECIMALES_TAN_PHI, valor)),
            )
        });
        base.into_iter().chain(tan_phi).collect()
    }
}

#[cfg(test)]
#[path = "../../tests/unit/inyeccion/dominio.rs"]
mod tests;
