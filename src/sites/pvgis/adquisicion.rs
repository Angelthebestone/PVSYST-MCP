//! PVGIS HTTP data acquisition.
//! Queries web services for horizon profiles and hourly TMY weather data.

use crate::adapters::http::http_get;
use crate::error::{Error, Result};
use serde_json::Value;

/// PVGIS 5.3 web service root.
pub(crate) const PVGIS_URL: &str = "https://re.jrc.ec.europa.eu/api/v5_3/";

/// Fetch the horizon profile for the given coordinates from PVGIS.
pub(crate) fn descargar_perfil_horizonte(lat: f64, lon: f64) -> Result<Vec<(f64, f64)>> {
    let url = format!("{PVGIS_URL}printhorizon?lat={lat}&lon={lon}&outputformat=json");
    let texto = http_get(&url)?;
    parsear_perfil_horizonte(&texto)
}

/// Parse horizon points `(A, H_hor)` from PVGIS `printhorizon` JSON, sorted by azimuth.
pub(crate) fn parsear_perfil_horizonte(texto: &str) -> Result<Vec<(f64, f64)>> {
    let v: Value = serde_json::from_str(texto)
        .map_err(|e| Error::Network(format!("Respuesta PVGIS no JSON: {e}")))?;
    let filas = v["outputs"]["horizon_profile"]
        .as_array()
        .ok_or_else(|| Error::Format("PVGIS sin outputs.horizon_profile".into()))?;
    let mut pts: Vec<(f64, f64)> = filas
        .iter()
        .filter_map(|f| Some((f["A"].as_f64()?, f["H_hor"].as_f64()?)))
        .collect();
    if pts.is_empty() {
        return Err(Error::Network("PVGIS devolvió un horizonte vacío".into()));
    }
    pts.sort_by(|a, b| a.0.total_cmp(&b.0));
    Ok(pts)
}

/// Fetch hourly TMY data for the given coordinates from PVGIS as raw JSON.
pub(crate) fn descargar_tmy(lat: f64, lon: f64) -> Result<String> {
    let url = format!("{PVGIS_URL}tmy?lat={lat}&lon={lon}&outputformat=json");
    http_get(&url)
}
