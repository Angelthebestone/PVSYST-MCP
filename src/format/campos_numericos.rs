//! The PVsyst fields this crate reads or writes as numbers.
//!
//! Writing text into one of them leaves a file PVsyst cannot read, so the
//! editors reject it. The list is deliberately not exhaustive: it holds the
//! fields this crate itself parses or formats as numbers, and no other field is
//! checked, because a free-text field may legitimately hold digits.

use crate::error::{Error, Result};

/// Numeric field names, compared case-insensitively.
const CAMPOS_NUMERICOS: &[&str] = &[
    // Module (.PAN) characteristics read by `components::pan`.
    "PNom",
    "NominalPower",
    "Voc",
    "Isc",
    "Impp",
    "Imp",
    "Vmpp",
    "Vmp",
    "muIsc",
    "muVocSpec",
    "muVoc",
    "muPnom",
    "muPmpp",
    "muPmpReq",
    "Gamma",
    "Gref",
    "Tref",
    // Variant and system fields written by the project use cases.
    "NModSerie",
    "NStringCh",
    "NStrOrient1",
    "PNomSizing",
    "NInverter",
    "NBattSerie",
    "NBattParall",
    "PLimGrid",
    "TanPhi",
    "BattSoCInit",
    "PMaxCharge",
    "PMaxDisch",
    "AxisAzim",
    "MinPhi",
    "MaxPhi",
    "Pitch",
    "CollectorWidth",
    "NSheds",
    // Losses.
    "SoilingLoss",
    "UFactor",
    "LIDLoss",
    "PMismMPP",
    // Economic evaluation.
    "ProjectLife",
    "DiscountRate",
    "InflationRate",
    "SummerPeakSaleTariff",
];

fn es_campo_numerico(campo: &str) -> bool {
    CAMPOS_NUMERICOS
        .iter()
        .any(|nombre| nombre.eq_ignore_ascii_case(campo))
}

/// PVsyst writes the decimal comma in some locales; both are numbers.
/// `NaN` and `inf` parse as floats but PVsyst cannot read them.
fn es_numero(valor: &str) -> bool {
    valor
        .trim()
        .replace(',', ".")
        .parse::<f64>()
        .is_ok_and(f64::is_finite)
}

/// Rejects a non-numeric value for a known numeric field.
pub(super) fn validar_valores(cambios: &[(String, Option<String>)]) -> Result<()> {
    for (campo, valor) in cambios {
        let Some(valor) = valor else {
            continue;
        };
        if es_campo_numerico(campo) && !es_numero(valor) {
            return Err(Error::Invalid(format!(
                "Valor no numérico '{valor}' para el campo numérico '{campo}'"
            )));
        }
    }
    Ok(())
}

#[cfg(test)]
#[path = "../../tests/unit/format/campos_numericos.rs"]
mod tests;
