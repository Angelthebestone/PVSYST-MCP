//! Input parameters and parsing for the easy-mode pipeline.

use super::lectura_argumentos::{
    parsear_bool_opcional, parsear_f64_opcional, parsear_f64_opcional_sin_defecto,
    parsear_f64_requerido, parsear_u32_opcional,
};
use crate::args::opt_str;
use crate::error::{Error, Result};
use crate::workspace;
use serde_json::Value;
use std::path::PathBuf;

pub(crate) const DEFAULT_TILT: f64 = 10.0;
pub(crate) const DEFAULT_AZIMUT: f64 = 0.0;
pub(crate) const DEFAULT_PLANTILLA: &str = "_DEMO_Residential";
pub(crate) const PLANTILLA_AISLADO: &str = "_DEMO_StandAlone";
pub(crate) const VARIANTE_RED: &str = "VC0";
pub(crate) const VARIANTE_AISLADO: &str = "VC3";
const TIPO_SISTEMA_RED: &str = "red";
const TIPO_SISTEMA_AISLADO: &str = "aislado";
pub(crate) const MIN_TILT: f64 = 0.0;
pub(crate) const MAX_TILT: f64 = 90.0;
pub(crate) const MIN_AZIMUT: f64 = -180.0;
pub(crate) const MAX_AZIMUT: f64 = 180.0;
pub(crate) const MIN_LATITUD: f64 = -90.0;
pub(crate) const MAX_LATITUD: f64 = 90.0;
pub(crate) const MIN_LONGITUD: f64 = -180.0;
pub(crate) const MAX_LONGITUD: f64 = 180.0;
pub(crate) const MIN_KWP: f64 = 0.0;

/// Design family: grid-connected or stand-alone.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum TipoDiseno {
    Red,
    Aislado,
}

impl TipoDiseno {
    fn parse(args: &Value) -> Result<Self> {
        let texto = opt_str(args, &["tipo_sistema"]).unwrap_or(TIPO_SISTEMA_RED);
        match texto.trim().to_lowercase().as_str() {
            TIPO_SISTEMA_RED => Ok(Self::Red),
            TIPO_SISTEMA_AISLADO => Ok(Self::Aislado),
            otro => Err(Error::Invalid(format!(
                "'tipo_sistema' debe ser 'red' o 'aislado'; se obtuvo '{otro}'"
            ))),
        }
    }

    pub(crate) fn nombre(self) -> &'static str {
        match self {
            Self::Red => TIPO_SISTEMA_RED,
            Self::Aislado => TIPO_SISTEMA_AISLADO,
        }
    }

    fn plantilla_por_defecto(self) -> &'static str {
        match self {
            Self::Red => DEFAULT_PLANTILLA,
            Self::Aislado => PLANTILLA_AISLADO,
        }
    }

    fn variante_por_defecto(self) -> &'static str {
        match self {
            Self::Red => VARIANTE_RED,
            Self::Aislado => VARIANTE_AISLADO,
        }
    }
}

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
    pub(crate) n_baterias_serie: Option<u32>,
    pub(crate) n_baterias_paralelo: Option<u32>,
    /// `via: "aproximacion"`: size with the MCP's own rules instead of PVsyst.
    pub(crate) via_aproximacion: bool,
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
    pub(crate) tipo_sistema: TipoDiseno,
    pub(crate) variante: String,
    pub(crate) ubicacion: UbicacionParametros,
    pub(crate) proyecto: ProyectoParametros,
    pub(crate) orientacion: OrientacionParametros,
    pub(crate) dimensionado: DimensionadoParametros,
    pub(crate) simulacion: SimulacionParametros,
    pub(crate) workspace: Option<PathBuf>,
}

impl UbicacionParametros {
    pub(crate) fn parse(args: &Value) -> Result<Self> {
        let latitud = parsear_f64_requerido(args, &["latitud"], "grados decimales")?;
        if !latitud.is_finite() || !(MIN_LATITUD..=MAX_LATITUD).contains(&latitud) {
            return Err(Error::Invalid(format!(
                "Latitud fuera de rango ({MIN_LATITUD} a {MAX_LATITUD}) o no finita: {latitud}"
            )));
        }
        let longitud = parsear_f64_requerido(args, &["longitud"], "grados decimales")?;
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
    pub(crate) fn parse(args: &Value, kwp: f64, tipo_sistema: TipoDiseno) -> Result<Self> {
        let nombre =
            opt_str(args, &["nombre"]).map_or_else(|| format!("Sistema_{kwp}kWp"), String::from);
        workspace::validar_nombre(&nombre)?;
        let plantilla = opt_str(args, &["plantilla"])
            .unwrap_or_else(|| tipo_sistema.plantilla_por_defecto())
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
        let tilt = parsear_f64_opcional(args, &["tilt"], DEFAULT_TILT)?;
        if !tilt.is_finite() || !(MIN_TILT..=MAX_TILT).contains(&tilt) {
            return Err(Error::Invalid(format!(
                "Inclinación (tilt) fuera de rango ({MIN_TILT} a {MAX_TILT}) o no finita: {tilt}"
            )));
        }
        let azimut = parsear_f64_opcional(args, &["azimut"], DEFAULT_AZIMUT)?;
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
        let n_baterias_serie = parsear_u32_opcional(args, &["n_baterias_serie"])?;
        let n_baterias_paralelo = parsear_u32_opcional(args, &["n_baterias_paralelo"])?;
        if [n_baterias_serie, n_baterias_paralelo]
            .into_iter()
            .flatten()
            .any(|n| n == 0)
        {
            return Err(Error::Invalid(
                "n_baterias_serie y n_baterias_paralelo deben ser mayores que cero".into(),
            ));
        }
        Ok(Self {
            kwp,
            modulo: opt_str(args, &["modulo"]).map(String::from),
            inversor: opt_str(args, &["inversor"]).map(String::from),
            pnom_modulo: parsear_f64_opcional_sin_defecto(args, &["pnom_modulo"])?,
            n_mod_serie: parsear_u32_opcional(args, &["n_mod_serie"])?,
            n_strings: parsear_u32_opcional(args, &["n_strings"])?,
            n_baterias_serie,
            n_baterias_paralelo,
            via_aproximacion: crate::aproximaciones::pide_aproximacion(args),
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
        let kwp = parsear_f64_requerido(args, &["kwp"], "potencia en kWp")?;
        if !kwp.is_finite() || kwp <= MIN_KWP {
            return Err(Error::Invalid(format!(
                "kwp debe ser un número finito mayor que cero: {kwp}"
            )));
        }
        let tipo_sistema = TipoDiseno::parse(args)?;
        let dimensionado = DimensionadoParametros::parse(args, kwp)?;
        if tipo_sistema == TipoDiseno::Red
            && (dimensionado.n_baterias_serie.is_some()
                || dimensionado.n_baterias_paralelo.is_some())
        {
            return Err(Error::Invalid(
                "n_baterias_serie y n_baterias_paralelo solo aplican a tipo_sistema 'aislado'"
                    .into(),
            ));
        }
        let variante = opt_str(args, &["variante"])
            .map(str::to_string)
            .unwrap_or_else(|| tipo_sistema.variante_por_defecto().to_string());
        Ok(Self {
            tipo_sistema,
            variante,
            ubicacion,
            proyecto: ProyectoParametros::parse(args, kwp, tipo_sistema)?,
            orientacion: OrientacionParametros::parse(args)?,
            dimensionado,
            simulacion: SimulacionParametros::parse(args)?,
            workspace: opt_str(args, &["workspace"]).map(PathBuf::from),
        })
    }
}
