//! One-axis tracker parameters and the `pvOrient` changes they require.

use crate::args::{opt_f64, opt_u32};
use crate::error::{Error, Result};
use serde_json::{json, Value};

pub(crate) const AZIMUT_EJE_DEFECTO: f64 = 0.0;
pub(crate) const ANGULO_MAX_DEFECTO: f64 = 60.0;
pub(crate) const PITCH_DEFECTO_M: f64 = 8.0;
pub(crate) const ANCHO_COLECTOR_DEFECTO_M: f64 = 4.25;
pub(crate) const N_FILAS_DEFECTO: u32 = 62;
const MIN_AZIMUT: f64 = -180.0;
const MAX_AZIMUT: f64 = 180.0;
const MAX_ANGULO: f64 = 90.0;
const DECIMALES_ANGULO: usize = 1;
const DECIMALES_METRO: usize = 3;

/// Whether the tracker uses backtracking.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Backtracking {
    Activo,
    Inactivo,
}

impl Backtracking {
    pub(crate) fn de_bool(activo: bool) -> Self {
        if activo {
            Self::Activo
        } else {
            Self::Inactivo
        }
    }

    fn valor(self) -> &'static str {
        match self {
            Self::Activo => "True",
            Self::Inactivo => "False",
        }
    }
}

/// Arguments that describe the tracker before validation.
#[derive(Debug, Clone, Copy)]
pub(crate) struct EntradaSeguidor {
    pub(crate) azimut_eje: Option<f64>,
    pub(crate) angulo_max: Option<f64>,
    pub(crate) pitch_m: Option<f64>,
    pub(crate) ancho_colector_m: Option<f64>,
    pub(crate) n_filas: Option<u32>,
    pub(crate) backtracking: Option<bool>,
}

impl EntradaSeguidor {
    pub(crate) fn de_args(args: &Value) -> Result<Self> {
        Ok(Self {
            azimut_eje: opt_f64(args, "azimut_eje")?,
            angulo_max: opt_f64(args, "angulo_max")?,
            pitch_m: opt_f64(args, "pitch_m")?,
            ancho_colector_m: opt_f64(args, "ancho_colector_m")?,
            n_filas: opt_u32(args, "n_filas")?,
            backtracking: args.get("backtracking").and_then(Value::as_bool),
        })
    }
}

/// Validated tracker configuration.
#[derive(Debug, Clone, PartialEq)]
pub(crate) struct ParametrosSeguidor {
    azimut_eje: f64,
    angulo_max: f64,
    pitch_m: f64,
    ancho_colector_m: f64,
    n_filas: u32,
    backtracking: Backtracking,
}

fn rango(valor: f64, (min, max): (f64, f64), clave: &str) -> Result<()> {
    if (min..=max).contains(&valor) {
        return Ok(());
    }
    Err(Error::Invalid(format!(
        "'{clave}' debe estar entre {min} y {max}: {valor}"
    )))
}

impl ParametrosSeguidor {
    pub(crate) fn nuevo(entrada: EntradaSeguidor) -> Result<Self> {
        let azimut_eje = entrada.azimut_eje.unwrap_or(AZIMUT_EJE_DEFECTO);
        rango(azimut_eje, (MIN_AZIMUT, MAX_AZIMUT), "azimut_eje")?;
        let angulo_max = entrada.angulo_max.unwrap_or(ANGULO_MAX_DEFECTO);
        if angulo_max <= 0.0 {
            return Err(Error::Invalid(format!(
                "'angulo_max' debe ser mayor que 0: {angulo_max}"
            )));
        }
        rango(angulo_max, (0.0, MAX_ANGULO), "angulo_max")?;
        let pitch_m = entrada.pitch_m.unwrap_or(PITCH_DEFECTO_M);
        let ancho_colector_m = entrada.ancho_colector_m.unwrap_or(ANCHO_COLECTOR_DEFECTO_M);
        if pitch_m <= ancho_colector_m || ancho_colector_m <= 0.0 {
            return Err(Error::Invalid(format!(
                "pitch_m ({pitch_m}) debe superar ancho_colector_m ({ancho_colector_m}), y ambos ser positivos"
            )));
        }
        let n_filas = entrada.n_filas.unwrap_or(N_FILAS_DEFECTO);
        if n_filas == 0 {
            return Err(Error::Invalid("'n_filas' debe ser al menos 1".into()));
        }
        Ok(Self {
            azimut_eje,
            angulo_max,
            pitch_m,
            ancho_colector_m,
            n_filas,
            backtracking: Backtracking::de_bool(entrada.backtracking.unwrap_or(true)),
        })
    }

    /// Arguments of the tracker in the shape the response reports them.
    pub(crate) fn como_json(&self) -> Value {
        json!({
            "azimut_eje": self.azimut_eje,
            "angulo_max": self.angulo_max,
            "pitch_m": self.pitch_m,
            "ancho_colector_m": self.ancho_colector_m,
            "n_filas": self.n_filas,
            "backtracking": matches!(self.backtracking, Backtracking::Activo),
        })
    }
}

/// `pvOrient` changes that turn a fixed plane into a one-axis tracker.
pub(crate) fn cambios_orientacion(
    p: &ParametrosSeguidor,
    no_orient: &str,
) -> Vec<(String, Option<String>)> {
    vec![
        ("NoOrient".into(), Some(no_orient.to_string())),
        (
            "AxisAzim".into(),
            Some(format!("{:.*}", DECIMALES_ANGULO, p.azimut_eje)),
        ),
        (
            "MinPhi".into(),
            Some(format!("{:.*}", DECIMALES_ANGULO, -p.angulo_max)),
        ),
        (
            "MaxPhi".into(),
            Some(format!("{:.*}", DECIMALES_ANGULO, p.angulo_max)),
        ),
        (
            "UsesBacktracking".into(),
            Some(p.backtracking.valor().to_string()),
        ),
        (
            "Pitch".into(),
            Some(format!("{:.*}", DECIMALES_METRO, p.pitch_m)),
        ),
        (
            "CollectorWidth".into(),
            Some(format!("{:.*}", DECIMALES_METRO, p.ancho_colector_m)),
        ),
        ("NSheds".into(), Some(p.n_filas.to_string())),
    ]
}

#[cfg(test)]
#[path = "../../tests/unit/seguidores/dominio.rs"]
mod tests;
