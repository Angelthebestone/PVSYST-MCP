//! Domain models and validation for PVsyst `.HOR` horizon profiles.

use crate::error::{Error, Result};
use serde::{Deserialize, Serialize};

/// Minimum valid azimuth in degrees.
pub(crate) const AZIMUT_MIN_GRADOS: f64 = -180.0;

/// Maximum valid azimuth in degrees.
pub(crate) const AZIMUT_MAX_GRADOS: f64 = 180.0;

/// Minimum valid height in degrees.
pub(crate) const ALTURA_MIN_GRADOS: f64 = 0.0;

/// Maximum valid height in degrees.
pub(crate) const ALTURA_MAX_GRADOS: f64 = 90.0;

/// Minimum points required for a valid horizon profile.
pub(crate) const PUNTOS_MINIMOS: usize = 2;

/// Maximum points supported by single-byte counter in TCubicProfile.
pub(crate) const PUNTOS_MAXIMOS: usize = 255;

/// A single point on the horizon profile.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub(crate) struct PuntoHorizonte {
    /// Azimuth angle in degrees (-180.0 to 180.0).
    #[serde(alias = "azimut_grados")]
    pub(crate) azimut: f64,
    /// Horizon height angle in degrees (0.0 to 90.0).
    #[serde(alias = "altura_grados")]
    pub(crate) altura: f64,
    /// Stored azimuth in radians if decoded from binary, preserving exact bits.
    #[serde(skip)]
    pub(crate) azimut_rad: Option<f64>,
    /// Stored height in radians if decoded from binary, preserving exact bits.
    #[serde(skip)]
    pub(crate) altura_rad: Option<f64>,
    /// Stored slope (dy/dx) if decoded from binary, preserving exact bits.
    #[serde(skip)]
    pub(crate) pendiente: Option<f64>,
}

impl PuntoHorizonte {
    /// Constructs a point from azimuth and height in degrees.
    #[must_use]
    pub(crate) fn de_grados(azimut: f64, altura: f64) -> Self {
        Self {
            azimut,
            altura,
            azimut_rad: None,
            altura_rad: None,
            pendiente: None,
        }
    }
}

/// Complete representation of a PVsyst `.HOR` horizon file.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub(crate) struct Horizonte {
    /// Title or comment describing the horizon line.
    pub(crate) titulo: String,
    /// Version string of the horizon format (typically "3.0").
    pub(crate) version: String,
    /// Calculation flag (typically "$0001").
    pub(crate) calcul: String,
    /// Ordered horizon points.
    pub(crate) puntos: Vec<PuntoHorizonte>,
}

/// Validates that horizon points meet PVsyst geometric and ordering constraints.
pub(crate) fn validar_puntos(puntos: &[PuntoHorizonte]) -> Result<()> {
    if puntos.len() < PUNTOS_MINIMOS {
        return Err(Error::Invalid(format!(
            "Horizonte inválido: se requieren al menos {PUNTOS_MINIMOS} puntos (recibidos {})",
            puntos.len()
        )));
    }
    if puntos.len() > PUNTOS_MAXIMOS {
        return Err(Error::Invalid(format!(
            "Horizonte inválido: máximo {PUNTOS_MAXIMOS} puntos (recibidos {})",
            puntos.len()
        )));
    }
    for p in puntos {
        validar_punto(p)?;
    }
    for i in 0..puntos.len() - 1 {
        if puntos[i + 1].azimut <= puntos[i].azimut {
            return Err(Error::Invalid(format!(
                "Puntos no ordenados por azimut: {:.2} seguido de {:.2}",
                puntos[i].azimut,
                puntos[i + 1].azimut
            )));
        }
    }
    Ok(())
}

fn validar_punto(p: &PuntoHorizonte) -> Result<()> {
    if !(AZIMUT_MIN_GRADOS..=AZIMUT_MAX_GRADOS).contains(&p.azimut) {
        return Err(Error::Invalid(format!(
            "Azimut fuera de rango [-180, 180]: {}",
            p.azimut
        )));
    }
    if !(ALTURA_MIN_GRADOS..=ALTURA_MAX_GRADOS).contains(&p.altura) {
        return Err(Error::Invalid(format!(
            "Altura fuera de rango [0, 90]: {}",
            p.altura
        )));
    }
    Ok(())
}
