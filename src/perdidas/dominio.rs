//! Variant loss parameters and the `pvMainArray` / `pvSubArray` changes.

use crate::args::{missing, opt_f64, rango_opcional};
use crate::error::Result;
use crate::format::edit::EdicionObjeto;
use serde_json::{json, Value};

const MAX_SUCIEDAD_PCT: f64 = 50.0;
const MAX_UC_TERMICO: f64 = 100.0;
const MAX_LID_PCT: f64 = 20.0;
const MAX_MISMATCH_PCT: f64 = 20.0;
const DECIMALES_SUCIEDAD: usize = 3;
const DECIMALES_UC: usize = 1;
const DECIMALES_PORCENTAJE: usize = 2;

/// Raw loss arguments; every field is optional.
#[derive(Debug, Clone, Copy, Default)]
pub(crate) struct EntradaPerdidas {
    pub(crate) suciedad_pct: Option<f64>,
    pub(crate) uc_termico: Option<f64>,
    pub(crate) lid_pct: Option<f64>,
    pub(crate) mismatch_pct: Option<f64>,
}

impl EntradaPerdidas {
    pub(crate) fn de_args(args: &Value) -> Result<Self> {
        Ok(Self {
            suciedad_pct: opt_f64(args, "suciedad_pct")?,
            uc_termico: opt_f64(args, "uc_termico")?,
            lid_pct: opt_f64(args, "lid_pct")?,
            mismatch_pct: opt_f64(args, "mismatch_pct")?,
        })
    }
}

/// Validated loss percentages.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub(crate) struct ParametrosPerdidas {
    suciedad_pct: Option<f64>,
    uc_termico: Option<f64>,
    lid_pct: Option<f64>,
    mismatch_pct: Option<f64>,
}

impl ParametrosPerdidas {
    pub(crate) fn nuevo(entrada: EntradaPerdidas) -> Result<Self> {
        if [
            entrada.suciedad_pct,
            entrada.uc_termico,
            entrada.lid_pct,
            entrada.mismatch_pct,
        ]
        .iter()
        .all(Option::is_none)
        {
            return Err(missing(
                "suciedad_pct",
                "o uc_termico, lid_pct, mismatch_pct",
            ));
        }
        rango_opcional(entrada.suciedad_pct, MAX_SUCIEDAD_PCT, "suciedad_pct")?;
        rango_opcional(entrada.uc_termico, MAX_UC_TERMICO, "uc_termico")?;
        rango_opcional(entrada.lid_pct, MAX_LID_PCT, "lid_pct")?;
        rango_opcional(entrada.mismatch_pct, MAX_MISMATCH_PCT, "mismatch_pct")?;
        Ok(Self {
            suciedad_pct: entrada.suciedad_pct,
            uc_termico: entrada.uc_termico,
            lid_pct: entrada.lid_pct,
            mismatch_pct: entrada.mismatch_pct,
        })
    }

    pub(crate) fn tiene_suciedad(&self) -> bool {
        self.suciedad_pct.is_some()
    }

    pub(crate) fn pide_subarrays(&self) -> bool {
        self.lid_pct.is_some() || self.mismatch_pct.is_some()
    }

    pub(crate) fn como_json(&self) -> Value {
        json!({
            "suciedad_pct": self.suciedad_pct,
            "uc_termico": self.uc_termico,
            "lid_pct": self.lid_pct,
            "mismatch_pct": self.mismatch_pct,
        })
    }

    fn main_array(&self) -> Option<EdicionObjeto> {
        let suciedad = self.suciedad_pct.map(|valor| {
            (
                "SoilingLoss".to_string(),
                Some(format!("{:.*}", DECIMALES_SUCIEDAD, valor / 100.0)),
            )
        });
        let termico = self.uc_termico.map(|valor| {
            (
                "UFactor".to_string(),
                Some(format!("{:.*}", DECIMALES_UC, valor)),
            )
        });
        let cambios: Vec<(String, Option<String>)> = suciedad.into_iter().chain(termico).collect();
        (!cambios.is_empty()).then(|| EdicionObjeto {
            selector: "pvMainArray".into(),
            cambios,
        })
    }

    fn subarrays(&self, n_subarrays: usize) -> Vec<EdicionObjeto> {
        (0..n_subarrays)
            .filter_map(|i| {
                let lid = self.lid_pct.map(|valor| {
                    (
                        "LIDLoss".to_string(),
                        Some(format!("{:.*}", DECIMALES_PORCENTAJE, valor)),
                    )
                });
                let mismatch = self.mismatch_pct.map(|valor| {
                    (
                        "PMismMPP".to_string(),
                        Some(format!("{:.*}", DECIMALES_PORCENTAJE, valor)),
                    )
                });
                let cambios: Vec<(String, Option<String>)> =
                    lid.into_iter().chain(mismatch).collect();
                (!cambios.is_empty()).then(|| EdicionObjeto {
                    selector: format!("pvSubArray[{i}]"),
                    cambios,
                })
            })
            .collect()
    }

    /// Object editions for the requested losses.
    pub(crate) fn ediciones(&self, n_subarrays: usize) -> Vec<EdicionObjeto> {
        self.main_array()
            .into_iter()
            .chain(self.subarrays(n_subarrays))
            .collect()
    }
}

/// PVsyst 8.1.7's newer soiling model, which this action deliberately refuses.
pub(crate) fn usa_formato_nuevo_suciedad(texto: &str) -> bool {
    texto.lines().any(|linea| linea.trim() == "Soiling=Soiling")
}

#[cfg(test)]
#[path = "../../tests/unit/perdidas/dominio.rs"]
mod tests;
