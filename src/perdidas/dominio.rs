//! Variant loss parameters and the `pvMainArray` / `pvSubArray` changes.

use crate::args::{missing, opt_f64, rango_opcional};
use crate::error::Result;
use crate::format::edit::EdicionObjeto;
use crate::format::span::{parse_spans, Lines, ObjSpan};
use crate::idioma::{Idioma, Texto};
use crate::perdidas::flags::{al_escribir_mismatch, FlagsMismatch};
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

/// What the variant already holds for one `pvSubArray`; the edits must respect it.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub(crate) struct SubarrayActual {
    /// `pvSubArray.Flags`, whose bits tell PVsyst to use its own default value.
    pub(crate) flags: Option<String>,
}

/// Every `pvSubArray` of the variant, in document order. The span parser already
/// splits the document for the edits, so this does not build a second full tree.
pub(crate) fn subarrays_de(bytes: &[u8]) -> Vec<SubarrayActual> {
    let lines = Lines::split(bytes);
    let spans = parse_spans(&lines);
    spans
        .iter()
        .filter(|span| span.ty.eq_ignore_ascii_case("pvSubArray"))
        .map(|span| SubarrayActual {
            flags: flags_de(&lines, &spans, span),
        })
        .collect()
}

/// `Flags` of the subarray itself: a line inside a nested object belongs to that
/// object, not to the subarray.
fn flags_de(lines: &Lines, spans: &[ObjSpan], span: &ObjSpan) -> Option<String> {
    let hijos: Vec<(usize, usize)> = spans
        .iter()
        .filter(|s| s.open > span.open && s.open < span.close)
        .map(|s| (s.open, s.close))
        .collect();
    (span.open + 1..span.close)
        .filter(|i| {
            !hijos
                .iter()
                .any(|(open, close)| (*open..=*close).contains(i))
        })
        .find_map(|i| {
            let texto = lines.text(i);
            let (clave, valor) = texto.trim_start().split_once('=')?;
            (clave.trim() == "Flags").then(|| valor.trim().to_string())
        })
}

/// Validated loss percentages.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub(crate) struct ParametrosPerdidas {
    suciedad_pct: Option<f64>,
    uc_termico: Option<f64>,
    lid_pct: Option<f64>,
    mismatch_pct: Option<f64>,
}

/// Validates the action's arguments without touching any file.
pub(crate) fn validar_parametros(args: &Value) -> Result<()> {
    ParametrosPerdidas::nuevo(EntradaPerdidas::de_args(args)?).map(|_| ())
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

    /// The `Flags` change that makes PVsyst read `PMismMPP`: without it the value
    /// is written and silently ignored.
    fn mismatch_legible(&self, actual: &SubarrayActual) -> Option<(String, Option<String>)> {
        self.mismatch_pct?;
        match al_escribir_mismatch(actual.flags.as_deref()) {
            FlagsMismatch::Limpiar(valor) => Some(("Flags".to_string(), Some(valor))),
            FlagsMismatch::YaLimpio | FlagsMismatch::Desconocido => None,
        }
    }

    fn subarrays(&self, subarrays: &[SubarrayActual]) -> Vec<EdicionObjeto> {
        subarrays
            .iter()
            .enumerate()
            .filter_map(|(i, actual)| {
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
                let cambios: Vec<(String, Option<String>)> = lid
                    .into_iter()
                    .chain(mismatch)
                    .chain(self.mismatch_legible(actual))
                    .collect();
                (!cambios.is_empty()).then(|| EdicionObjeto {
                    selector: format!("pvSubArray[{i}]"),
                    cambios,
                })
            })
            .collect()
    }

    /// Object editions for the requested losses.
    pub(crate) fn ediciones(&self, subarrays: &[SubarrayActual]) -> Vec<EdicionObjeto> {
        self.main_array()
            .into_iter()
            .chain(self.subarrays(subarrays))
            .collect()
    }

    /// Subarrays whose `Flags` could not be read, so `PMismMPP` may stay ignored.
    pub(crate) fn avisos(&self, subarrays: &[SubarrayActual]) -> Vec<String> {
        if self.mismatch_pct.is_none() {
            return Vec::new();
        }
        subarrays
            .iter()
            .enumerate()
            .filter(|(_, actual)| {
                matches!(
                    al_escribir_mismatch(actual.flags.as_deref()),
                    FlagsMismatch::Desconocido
                )
            })
            .map(|(i, actual)| {
                FLAGS_ILEGIBLE.texto_con(
                    Idioma::de_entorno(),
                    &[
                        ("i", &i.to_string()),
                        ("flags", actual.flags.as_deref().unwrap_or("")),
                    ],
                )
            })
            .collect()
    }
}

/// A `Flags` value this code cannot rewrite, so PVsyst may keep ignoring `PMismMPP`.
const FLAGS_ILEGIBLE: Texto = Texto {
    es: "pvSubArray[{i}].Flags='{flags}' no es un valor hexadecimal con $: PVsyst puede seguir ignorando PMismMPP",
    en: "pvSubArray[{i}].Flags='{flags}' is not a $-hex value: PVsyst may still ignore PMismMPP",
};

/// PVsyst 8.1.7's newer soiling model, which this action deliberately refuses.
pub(crate) fn usa_formato_nuevo_suciedad(texto: &str) -> bool {
    texto.lines().any(|linea| linea.trim() == "Soiling=Soiling")
}

#[cfg(test)]
#[path = "../../tests/unit/perdidas/dominio.rs"]
mod tests;
