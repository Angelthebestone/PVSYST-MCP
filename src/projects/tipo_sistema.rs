//! System type of a project variant, read from `pvSystem.SystemType`.

#![cfg_attr(
    not(test),
    expect(
        dead_code,
        reason = "consumed by the follower, injection and storage actions (3.3-3.5)"
    )
)]

use crate::error::{Error, Result};
use crate::format::object;

/// PVsyst's four project system kinds.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum TipoSistema {
    Red,
    Aislado,
    Bombeo,
    RedDc,
}

impl TipoSistema {
    fn nombre(self) -> &'static str {
        match self {
            Self::Red => "Red",
            Self::Aislado => "Aislado",
            Self::Bombeo => "Bombeo",
            Self::RedDc => "Red DC",
        }
    }

    /// Parse the variant's `SystemType` field.
    ///
    /// Unknown or missing values are `Error::Format`: the file cannot be
    /// interpreted, as opposed to a valid variant of the wrong kind.
    pub(crate) fn de_variante(bytes: &[u8]) -> Result<Self> {
        let texto = String::from_utf8_lossy(bytes);
        let raiz = object::parse_first(&texto)
            .ok_or_else(|| Error::Format("La variante no contiene un PVObject raíz".into()))?;
        let valor = raiz
            .find_deep("SystemType")
            .ok_or_else(|| Error::Format("La variante no declara SystemType".into()))?
            .trim();
        match valor.to_ascii_lowercase().as_str() {
            "grid" => Ok(Self::Red),
            "battery" => Ok(Self::Aislado),
            "pumping" => Ok(Self::Bombeo),
            "dcgrid" => Ok(Self::RedDc),
            _ => Err(Error::Format(format!(
                "Tipo de sistema desconocido: '{valor}'"
            ))),
        }
    }

    /// Require the variant to be of `esperado`.
    pub(crate) fn exigir(bytes: &[u8], esperado: Self) -> Result<()> {
        let real = Self::de_variante(bytes)?;
        if real == esperado {
            return Ok(());
        }
        Err(Error::Invalid(format!(
            "La variante es de tipo {}; esta acción requiere {}",
            real.nombre(),
            esperado.nombre()
        )))
    }
}

#[cfg(test)]
#[path = "../../tests/unit/projects/tipo_sistema.rs"]
mod tests;
