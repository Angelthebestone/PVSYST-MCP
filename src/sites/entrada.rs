//! Boundary checks for site and weather-data inputs. Every check runs before any
//! NASA lookup or PVsystCLI call, and each error names the offending parameter.

use crate::args::req_numero;
use crate::error::{Error, Result};
use crate::sites::solar;
use serde_json::Value;
use std::path::Path;

/// Continents accepted by the `region` parameter (PVsystCLI `-r`).
const REGIONES: &[&str] = &[
    "africa",
    "antarctica",
    "asia",
    "australia",
    "europe",
    "north_america",
    "pacific",
    "south_america",
];
/// Allowed `input_mef_timeshift` range in hours (PVsystCLI `-imt`).
const TIMESHIFT_MINIMO: f64 = -30.0;
const TIMESHIFT_MAXIMO: f64 = 30.0;

/// Rejects a `region` value that PVsystCLI does not know.
pub(crate) fn validar_region(args: &Value) -> Result<()> {
    match args.get("region").and_then(Value::as_str) {
        Some(region) if !region.is_empty() && !REGIONES.contains(&region) => {
            Err(Error::Invalid(format!(
                "'region' desconocida '{region}': use una de {}",
                REGIONES.join(", ")
            )))
        }
        _ => Ok(()),
    }
}

/// Rejects latitude or longitude outside the geographic limits.
pub(crate) fn validar_coordenadas(lat: f64, lon: f64) -> Result<()> {
    rango(
        "latitud",
        lat,
        (solar::LATITUD_MINIMA, solar::LATITUD_MAXIMA),
    )?;
    rango(
        "longitud",
        lon,
        (solar::LONGITUD_MINIMA, solar::LONGITUD_MAXIMA),
    )
}

fn rango(parametro: &str, valor: f64, (minimo, maximo): (f64, f64)) -> Result<()> {
    if (minimo..=maximo).contains(&valor) {
        return Ok(());
    }
    Err(Error::Invalid(format!(
        "'{parametro}' fuera de rango ({minimo} a {maximo}): {valor}"
    )))
}

/// Validates `input_mef_timeshift` when given: a number within [-30, 30].
pub(crate) fn validar_timeshift(args: &Value) -> Result<()> {
    if args.get("input_mef_timeshift").is_none_or(Value::is_null) {
        return Ok(());
    }
    rango(
        "input_mef_timeshift",
        req_numero(args, "input_mef_timeshift", "")?,
        (TIMESHIFT_MINIMO, TIMESHIFT_MAXIMO),
    )
}

/// Rejects an input file that does not exist; the error names the parameter.
pub(crate) fn exigir_archivo(parametro: &str, ruta: &Path) -> Result<()> {
    if ruta.is_file() {
        return Ok(());
    }
    Err(Error::Invalid(format!(
        "'{parametro}' no existe: {}",
        ruta.display()
    )))
}
