//! Domain models and physical conversion logic for PVsyst hourly series.
//!
//! Enforces physical boundary conditions and scales between stored raw integers
//! and standard engineering units (W/m², °C, m/s, kWh, etc.).

use crate::error::{Error, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

pub(crate) const HORAS_ANO_ESTANDAR: usize = 8760;
pub(crate) const HORAS_ANO_BISIESTO: usize = 8784;
const FACTOR_IRRAD_SI_A_HORA: f64 = 1000.0 / 3.6;
const FACTOR_IRRAD_HORA_A_SI: f64 = 3.6 / 1000.0;
const FACTOR_ENERGIA_MJ_A_KWH: f64 = 1.0 / 3.6;
const FACTOR_ENERGIA_KWH_A_MJ: f64 = 3.6;
const FACTOR_DEFECTO_IRRAD: f64 = 2777.778;
const FACTOR_DEFECTO_TEMPER: f64 = 100.0;
const FACTOR_DEFECTO_VIENTO: f64 = 100.0;
const FACTOR_DEFECTO_FACTOR: f64 = 2000.0;
const FACTOR_DEFECTO_ENERGIA: f64 = 277.7778;
const FACTOR_DEFECTO_TENSION: f64 = 100.0;
const FACTOR_DEFECTO_CORRIENTE: f64 = 500.0;

/// Variable measurement type according to PVsyst conventions.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) enum TipoVariable {
    Irradiacion,
    Temperatura,
    Viento,
    Factor,
    Energia,
    Tension,
    Corriente,
    Otro,
}

impl TipoVariable {
    /// Informs the physical variable type from metadata fields.
    pub(crate) fn de_metadatos(tipo: &str, nombre: &str) -> Self {
        match tipo {
            "Irrad" => Self::Irradiacion,
            "Temper" => Self::Temperatura,
            "WindVel" => Self::Viento,
            "Factor" => Self::Factor,
            "Energy" => Self::Energia,
            "Voltage" => Self::Tension,
            "Current" => Self::Corriente,
            _ => match nombre {
                "GlobHor" | "DiffHor" | "GlPMeas" => Self::Irradiacion,
                "T_Amb" | "TArrMes" => Self::Temperatura,
                "WindVel" => Self::Viento,
                "RelHum" | "Linke" => Self::Factor,
                _ => Self::Otro,
            },
        }
    }

    /// Measurement unit name for display.
    pub(crate) fn unidad(&self) -> &'static str {
        match self {
            Self::Irradiacion => "W/m²",
            Self::Temperatura => "°C",
            Self::Viento => "m/s",
            Self::Factor => "fraccion",
            Self::Energia => "kWh",
            Self::Tension => "V",
            Self::Corriente => "A",
            Self::Otro => "u",
        }
    }

    /// Default WRFactor used when writing variables without existing factor.
    pub(crate) fn wr_factor_defecto(&self) -> f64 {
        match self {
            Self::Irradiacion => FACTOR_DEFECTO_IRRAD,
            Self::Temperatura => FACTOR_DEFECTO_TEMPER,
            Self::Viento => FACTOR_DEFECTO_VIENTO,
            Self::Factor => FACTOR_DEFECTO_FACTOR,
            Self::Energia => FACTOR_DEFECTO_ENERGIA,
            Self::Tension => FACTOR_DEFECTO_TENSION,
            Self::Corriente => FACTOR_DEFECTO_CORRIENTE,
            Self::Otro => FACTOR_DEFECTO_TEMPER,
        }
    }
}

/// Metadata stored in each `VarSimul` block.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub(crate) struct MetadatosVariable {
    pub(crate) nombre: String,
    pub(crate) tipo: TipoVariable,
    pub(crate) wr_factor: f64,
    pub(crate) indice: usize,
    pub(crate) header_lines: Vec<String>,
}

/// Hourly measurement record containing all variable values.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub(crate) struct HoraValores {
    pub(crate) hora: usize,
    pub(crate) dia: String,
    pub(crate) valores: HashMap<String, f64>,
}

/// Full decoded hourly dataset from .MET or .DAM body.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub(crate) struct SerieHoraria {
    pub(crate) top_line: String,
    pub(crate) variables: Vec<String>,
    pub(crate) metadatos: Vec<MetadatosVariable>,
    pub(crate) dias: Vec<String>,
    pub(crate) horas: Vec<HoraValores>,
    #[serde(skip)]
    pub(crate) valores_crudos: HashMap<String, Vec<i64>>,
}

/// Allowed physical range for input validation.
#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) struct RangoFisico {
    pub(crate) min: f64,
    pub(crate) max: f64,
}

/// Converts raw stored integer to physical engineering unit.
pub(crate) fn crudo_a_fisico(raw: i64, wr_factor: f64, tipo: TipoVariable) -> f64 {
    let r = raw as f64;
    let factor = if wr_factor > 0.0 {
        wr_factor
    } else {
        tipo.wr_factor_defecto()
    };
    match tipo {
        TipoVariable::Irradiacion => (r / factor) * FACTOR_IRRAD_SI_A_HORA,
        TipoVariable::Energia => (r / factor) * FACTOR_ENERGIA_MJ_A_KWH,
        TipoVariable::Temperatura
        | TipoVariable::Viento
        | TipoVariable::Factor
        | TipoVariable::Tension
        | TipoVariable::Corriente
        | TipoVariable::Otro => r / factor,
    }
}

/// Converts physical engineering value to raw stored integer.
pub(crate) fn fisico_a_crudo(fisico: f64, wr_factor: f64, tipo: TipoVariable) -> i64 {
    let factor = if wr_factor > 0.0 {
        wr_factor
    } else {
        tipo.wr_factor_defecto()
    };
    match tipo {
        TipoVariable::Irradiacion => (fisico * FACTOR_IRRAD_HORA_A_SI * factor).round() as i64,
        TipoVariable::Energia => (fisico * FACTOR_ENERGIA_KWH_A_MJ * factor).round() as i64,
        TipoVariable::Temperatura
        | TipoVariable::Viento
        | TipoVariable::Factor
        | TipoVariable::Tension
        | TipoVariable::Corriente
        | TipoVariable::Otro => (fisico * factor).round() as i64,
    }
}

/// Returns expected physical ranges for validation.
pub(crate) fn rango_valido(nombre: &str, tipo: TipoVariable) -> RangoFisico {
    match tipo {
        TipoVariable::Irradiacion => RangoFisico {
            min: 0.0,
            max: 1500.0,
        },
        TipoVariable::Temperatura => RangoFisico {
            min: -80.0,
            max: 75.0,
        },
        TipoVariable::Viento => RangoFisico {
            min: 0.0,
            max: 100.0,
        },
        TipoVariable::Factor => RangoFisico {
            min: 0.0,
            max: 100.0,
        },
        TipoVariable::Energia => RangoFisico {
            min: 0.0,
            max: 10_000_000.0,
        },
        TipoVariable::Tension => RangoFisico {
            min: 0.0,
            max: 5000.0,
        },
        TipoVariable::Corriente => RangoFisico {
            min: 0.0,
            max: 5000.0,
        },
        TipoVariable::Otro => match nombre {
            "GlobHor" | "DiffHor" | "GlPMeas" => RangoFisico {
                min: 0.0,
                max: 1500.0,
            },
            "T_Amb" | "TArrMes" => RangoFisico {
                min: -80.0,
                max: 75.0,
            },
            "WindVel" => RangoFisico {
                min: 0.0,
                max: 100.0,
            },
            _ => RangoFisico {
                min: -1_000_000.0,
                max: 1_000_000.0,
            },
        },
    }
}

/// Validates hourly series length and physical bounds per variable.
pub(crate) fn validar_serie(variables: &[String], valores_por_variable: &[Vec<f64>]) -> Result<()> {
    if variables.is_empty() {
        return Err(Error::Invalid("Lista de variables vacia".into()));
    }
    if variables.len() != valores_por_variable.len() {
        return Err(Error::Invalid(format!(
            "Cantidad de variables ({}) no coincide con columnas de valores ({})",
            variables.len(),
            valores_por_variable.len()
        )));
    }
    for (nombre, vals) in variables.iter().zip(valores_por_variable.iter()) {
        let n = vals.len();
        if n != HORAS_ANO_ESTANDAR && n != HORAS_ANO_BISIESTO {
            return Err(Error::Invalid(format!(
                "Variable '{nombre}' tiene {n} horas; esperado {HORAS_ANO_ESTANDAR} o {HORAS_ANO_BISIESTO}"
            )));
        }
        let tipo = TipoVariable::de_metadatos("", nombre);
        let rango = rango_valido(nombre, tipo);
        for (h, &val) in vals.iter().enumerate() {
            if val < rango.min || val > rango.max {
                return Err(Error::Invalid(format!(
                    "Valor {val} fuera de rango fisico [{min}, {max}] para '{nombre}' en hora {h}",
                    min = rango.min,
                    max = rango.max
                )));
            }
        }
    }
    Ok(())
}
