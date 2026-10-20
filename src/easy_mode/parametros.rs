//! Input parameters and parsing for the easy-mode pipeline.

use crate::args::{missing, opt_str};
use crate::error::{Error, Result};
use crate::workspace;
use serde_json::Value;
use std::path::PathBuf;

pub(crate) const DEFAULT_TILT: f64 = 10.0;
pub(crate) const DEFAULT_AZIMUT: f64 = 0.0;
pub(crate) const DEFAULT_PLANTILLA: &str = "_DEMO_Residential";
pub(crate) const MIN_TILT: f64 = 0.0;
pub(crate) const MAX_TILT: f64 = 90.0;
pub(crate) const MIN_AZIMUT: f64 = -180.0;
pub(crate) const MAX_AZIMUT: f64 = 180.0;
pub(crate) const MIN_LATITUD: f64 = -90.0;
pub(crate) const MAX_LATITUD: f64 = 90.0;
pub(crate) const MIN_LONGITUD: f64 = -180.0;
pub(crate) const MAX_LONGITUD: f64 = 180.0;
pub(crate) const MIN_KWP: f64 = 0.0;

/// Validated geographic location parameters.
#[derive(Debug, Clone)]
pub(crate) struct UbicacionParametros {
    pub(crate) latitud: f64,
    pub(crate) longitud: f64,
    pub(crate) altitud: Option<f64>,
    pub(crate) timezone: Option<f64>,
    pub(crate) country_code: Option<String>,
    pub(crate) region: Option<String>,
}

/// Project creation and lifecycle parameters.
#[derive(Debug, Clone)]
pub(crate) struct ProyectoParametros {
    pub(crate) nombre: String,
    pub(crate) plantilla: String,
    pub(crate) sobrescribir: bool,
}

/// Fixed plane orientation parameters in degrees.
#[derive(Debug, Clone)]
pub(crate) struct OrientacionParametros {
    pub(crate) tilt: f64,
    pub(crate) azimut: f64,
}

/// Array sizing and electrical design parameters.
#[derive(Debug, Clone)]
pub(crate) struct DimensionadoParametros {
    pub(crate) kwp: f64,
    pub(crate) modulo: Option<String>,
    pub(crate) inversor: Option<String>,
    pub(crate) pnom_modulo: Option<f64>,
    pub(crate) n_mod_serie: Option<u32>,
    pub(crate) n_strings: Option<u32>,
}

/// Simulation execution and report parameters.
#[derive(Debug, Clone)]
pub(crate) struct SimulacionParametros {
    pub(crate) simular: bool,
    pub(crate) synthetic_weather: bool,
    pub(crate) output_csv: Option<String>,
    pub(crate) report_pdf: Option<String>,
}

/// Parsed and strictly validated end-to-end design request.
#[derive(Debug, Clone)]
pub(crate) struct DisenoParametros {
    pub(crate) ubicacion: UbicacionParametros,
    pub(crate) proyecto: ProyectoParametros,
    pub(crate) orientacion: OrientacionParametros,
    pub(crate) dimensionado: DimensionadoParametros,
    pub(crate) simulacion: SimulacionParametros,
    pub(crate) workspace: Option<PathBuf>,
}

pub(crate) fn parsear_f64_requerido(args: &Value, claves: &[&str], pista: &str) -> Result<f64> {
    parsear_f64_opcional_sin_defecto(args, claves)?.ok_or_else(|| missing(claves[0], pista))
}

pub(crate) fn parsear_f64_opcional(args: &Value, claves: &[&str], defecto: f64) -> Result<f64> {
    Ok(parsear_f64_opcional_sin_defecto(args, claves)?.unwrap_or(defecto))
}

pub(crate) fn parsear_f64_opcional_sin_defecto(
    args: &Value,
    claves: &[&str],
) -> Result<Option<f64>> {
    for clave in claves {
        if let Some(valor) = args.get(*clave) {
            if valor.is_null() {
                continue;
            }
            if let Some(n) = valor.as_f64() {
                if !n.is_finite() {
                    return Err(Error::Invalid(format!(
                        "Valor no finito para '{clave}': {n}"
                    )));
                }
                return Ok(Some(n));
            }
            if let Some(s) = valor.as_str() {
                let recortado = s.trim();
                if recortado.is_empty() {
                    continue;
                }
                let parsed = recortado.parse::<f64>().map_err(|_| {
                    Error::Invalid(format!(
                        "Valor no numérico para '{clave}': se esperaba un número, se obtuvo '{s}'"
                    ))
                })?;
                if !parsed.is_finite() {
                    return Err(Error::Invalid(format!(
                        "Valor no finito para '{clave}': {parsed}"
                    )));
                }
                return Ok(Some(parsed));
            }
            return Err(Error::Invalid(format!(
                "Tipo no válido para '{clave}': se esperaba un número"
            )));
        }
    }
    Ok(None)
}

pub(crate) fn parsear_u32_opcional(args: &Value, claves: &[&str]) -> Result<Option<u32>> {
    for clave in claves {
        if let Some(valor) = args.get(*clave) {
            if valor.is_null() {
                continue;
            }
            if let Some(n) = valor.as_u64() {
                let u = u32::try_from(n).map_err(|_| {
                    Error::Invalid(format!(
                        "Valor entero fuera de rango para '{clave}': '{n}' excede el límite u32 ({})",
                        u32::MAX
                    ))
                })?;
                return Ok(Some(u));
            }
            if let Some(i) = valor.as_i64() {
                // `as_u64` failed, so the integer is negative.
                return Err(Error::Invalid(format!(
                    "Valor entero para '{clave}' no puede ser negativo: {i}"
                )));
            }
            if let Some(s) = valor.as_str() {
                let recortado = s.trim();
                if recortado.is_empty() {
                    continue;
                }
                let parsed = recortado.parse::<u32>().map_err(|_| {
                    Error::Invalid(format!("Valor entero no válido para '{clave}': '{s}'"))
                })?;
                return Ok(Some(parsed));
            }
            return Err(Error::Invalid(format!(
                "Tipo no válido para '{clave}': se esperaba un entero positivo"
            )));
        }
    }
    Ok(None)
}

pub(crate) fn parsear_bool_opcional(args: &Value, claves: &[&str], defecto: bool) -> Result<bool> {
    for clave in claves {
        if let Some(valor) = args.get(*clave) {
            if valor.is_null() {
                continue;
            }
            if let Some(b) = valor.as_bool() {
                return Ok(b);
            }
            if let Some(s) = valor.as_str() {
                match s.trim().to_lowercase().as_str() {
                    "true" | "1" | "si" | "yes" => return Ok(true),
                    "false" | "0" | "no" => return Ok(false),
                    _ => {
                        return Err(Error::Invalid(format!(
                            "Valor booleano no válido para '{clave}': '{s}'"
                        )))
                    }
                }
            }
            return Err(Error::Invalid(format!(
                "Tipo no válido para '{clave}': se esperaba un booleano"
            )));
        }
    }
    Ok(defecto)
}

impl UbicacionParametros {
    pub(crate) fn parse(args: &Value) -> Result<Self> {
        let latitud = parsear_f64_requerido(args, &["latitud", "latitude"], "grados decimales")?;
        if !latitud.is_finite() || !(MIN_LATITUD..=MAX_LATITUD).contains(&latitud) {
            return Err(Error::Invalid(format!(
                "Latitud fuera de rango ({MIN_LATITUD} a {MAX_LATITUD}) o no finita: {latitud}"
            )));
        }
        let longitud = parsear_f64_requerido(args, &["longitud", "longitude"], "grados decimales")?;
        if !longitud.is_finite() || !(MIN_LONGITUD..=MAX_LONGITUD).contains(&longitud) {
            return Err(Error::Invalid(format!(
                "Longitud fuera de rango ({MIN_LONGITUD} a {MAX_LONGITUD}): {longitud}"
            )));
        }
        Ok(Self {
            latitud,
            longitud,
            altitud: parsear_f64_opcional_sin_defecto(args, &["altitud"])?,
            timezone: parsear_f64_opcional_sin_defecto(args, &["timezone"])?,
            country_code: opt_str(args, &["country_code"]).map(String::from),
            region: opt_str(args, &["region"]).map(String::from),
        })
    }
}

impl ProyectoParametros {
    pub(crate) fn parse(args: &Value, kwp: f64) -> Result<Self> {
        let nombre =
            opt_str(args, &["nombre"]).map_or_else(|| format!("Sistema_{kwp}kWp"), String::from);
        workspace::validar_nombre(&nombre)?;
        let plantilla = opt_str(args, &["plantilla"])
            .unwrap_or(DEFAULT_PLANTILLA)
            .to_string();
        let sobrescribir = parsear_bool_opcional(args, &["sobrescribir"], false)?;
        Ok(Self {
            nombre,
            plantilla,
            sobrescribir,
        })
    }
}

impl OrientacionParametros {
    pub(crate) fn parse(args: &Value) -> Result<Self> {
        let tilt = parsear_f64_opcional(args, &["tilt", "inclinacion"], DEFAULT_TILT)?;
        if !tilt.is_finite() || !(MIN_TILT..=MAX_TILT).contains(&tilt) {
            return Err(Error::Invalid(format!(
                "Inclinación (tilt) fuera de rango ({MIN_TILT} a {MAX_TILT}) o no finita: {tilt}"
            )));
        }
        let azimut = parsear_f64_opcional(args, &["azimut", "azimuth"], DEFAULT_AZIMUT)?;
        if !azimut.is_finite() || !(MIN_AZIMUT..=MAX_AZIMUT).contains(&azimut) {
            return Err(Error::Invalid(format!(
                "Azimut fuera de rango ({MIN_AZIMUT} a {MAX_AZIMUT}) o no finito: {azimut}"
            )));
        }
        Ok(Self { tilt, azimut })
    }
}

impl DimensionadoParametros {
    pub(crate) fn parse(args: &Value, kwp: f64) -> Result<Self> {
        Ok(Self {
            kwp,
            modulo: opt_str(args, &["modulo"]).map(String::from),
            inversor: opt_str(args, &["inversor"]).map(String::from),
            pnom_modulo: parsear_f64_opcional_sin_defecto(args, &["pnom_modulo"])?,
            n_mod_serie: parsear_u32_opcional(args, &["n_mod_serie"])?,
            n_strings: parsear_u32_opcional(args, &["n_strings"])?,
        })
    }
}

impl SimulacionParametros {
    pub(crate) fn parse(args: &Value) -> Result<Self> {
        Ok(Self {
            simular: parsear_bool_opcional(args, &["simular"], true)?,
            synthetic_weather: parsear_bool_opcional(args, &["synthetic_weather"], true)?,
            output_csv: opt_str(args, &["output_csv"]).map(String::from),
            report_pdf: opt_str(args, &["report_pdf"]).map(String::from),
        })
    }
}

impl DisenoParametros {
    /// Validates and parses untyped JSON arguments into structured design parameters.
    pub(crate) fn parse(args: &Value) -> Result<Self> {
        let ubicacion = UbicacionParametros::parse(args)?;
        let kwp = parsear_f64_requerido(args, &["kwp", "potencia_kwp"], "potencia en kWp")?;
        if !kwp.is_finite() || kwp <= MIN_KWP {
            return Err(Error::Invalid(format!(
                "kwp debe ser un número finito mayor que cero: {kwp}"
            )));
        }
        Ok(Self {
            ubicacion,
            proyecto: ProyectoParametros::parse(args, kwp)?,
            orientacion: OrientacionParametros::parse(args)?,
            dimensionado: DimensionadoParametros::parse(args, kwp)?,
            simulacion: SimulacionParametros::parse(args)?,
            workspace: opt_str(args, &["workspace"]).map(PathBuf::from),
        })
    }
}
