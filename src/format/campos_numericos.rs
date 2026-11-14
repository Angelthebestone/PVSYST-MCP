//! The PVsyst fields this crate reads or writes as numbers.
//!
//! Writing text into one of them leaves a file PVsyst cannot read, so the
//! editors reject it. The list is deliberately not exhaustive: it holds the
//! fields this crate itself parses or formats as numbers, and no other field is
//! checked, because a free-text field may legitimately hold digits.

use crate::error::{Error, Result};
use crate::idioma::{Idioma, Texto};

/// Refusal of text in a field PVsyst reads as a number.
const VALOR_NO_NUMERICO: Texto = Texto {
    es: "Valor no numérico '{valor}' para el campo numérico '{campo}'",
    en: "Non-numeric value '{valor}' for the numeric field '{campo}'",
};

/// Refusal of a decimal comma: PVsyst writes the point, and in some locales the
/// comma is a thousands separator, so `1,234` would mean 1234 to the user.
const VALOR_CON_COMA: Texto = Texto {
    es: "Valor '{valor}' para el campo numérico '{campo}': PVsyst escribe el decimal con punto, no con coma",
    en: "Value '{valor}' for the numeric field '{campo}': PVsyst writes the decimal point, not a comma",
};

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
    "muGamma",
    "Gref",
    "Tref",
    "NCelS",
    "RShunt",
    "RSerie",
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

/// PVsyst writes numbers with a decimal point. `NaN` and `inf` parse as floats
/// but PVsyst cannot read them, so they are not numbers here either.
///
/// The write path is strict on purpose: a comma is refused before this check, while
/// `format::numero_texto` accepts it for tolerant comparisons. Do not "fix" one side
/// without the other.
fn es_numero(valor: &str) -> bool {
    valor.trim().parse::<f64>().is_ok_and(f64::is_finite)
}

/// Rejects a non-numeric value for a known numeric field.
pub(crate) fn validar_valores(cambios: &[(String, Option<String>)]) -> Result<()> {
    for (campo, valor) in cambios {
        let Some(valor) = valor else {
            continue;
        };
        if !es_campo_numerico(campo) {
            continue;
        }
        if valor.contains(',') {
            return Err(Error::Invalid(VALOR_CON_COMA.texto_con(
                Idioma::de_entorno(),
                &[("valor", valor), ("campo", campo)],
            )));
        }
        if !es_numero(valor) {
            return Err(Error::Invalid(VALOR_NO_NUMERICO.texto_con(
                Idioma::de_entorno(),
                &[("valor", valor), ("campo", campo)],
            )));
        }
    }
    Ok(())
}

#[cfg(test)]
#[path = "../../tests/unit/format/campos_numericos.rs"]
mod tests;
