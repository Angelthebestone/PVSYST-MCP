use super::caso::{caso, con_cabecera, round1, Caso};
use super::supuestos::{acotado, Rango};
use crate::adapters::http::http_get;
use crate::args::flag;
use crate::config::Paths;
use crate::error::{Error, Result};
use serde_json::{json, Value};
use std::collections::BTreeMap;

pub(super) const SIGMA_METEO_DEFECTO: f64 = 5.0;
pub(super) const SIGMA_MODELO_DEFECTO: f64 = 3.0;
pub(super) const Z_P90: f64 = 1.2816;
pub(super) const Z_P99: f64 = 2.3263;
const SIGMA_METEO: Rango = Rango {
    clave: "sigma_meteo_pct",
    min: 0.0,
    max: 100.0,
};
const SIGMA_MODELO: Rango = Rango {
    clave: "sigma_modelo_pct",
    min: 0.0,
    max: 100.0,
};

pub(super) fn coef_variacion(xs: &[f64]) -> Option<f64> {
    if xs.len() < 2 {
        return None;
    }
    let n = xs.len() as f64;
    let media = xs.iter().sum::<f64>() / n;
    if media <= 0.0 {
        return None;
    }
    let var = xs.iter().map(|x| (x - media).powi(2)).sum::<f64>() / (n - 1.0);
    Some(var.sqrt() / media)
}

pub(super) fn sigma_meteo_pvgis(c: &Caso) -> Result<f64> {
    let sitio = c.sitio()?;
    let lat = sitio
        .lat
        .ok_or_else(|| Error::Format("PRJ sin latitud".into()))?;
    let lon = sitio
        .lon
        .ok_or_else(|| Error::Format("PRJ sin longitud".into()))?;
    let url = format!(
        "https://re.jrc.ec.europa.eu/api/v5_3/MRcalc?lat={lat}&lon={lon}&horirrad=1&outputformat=json"
    );
    let cuerpo: Value = serde_json::from_str(&http_get(&url)?)
        .map_err(|e| Error::Invalid(format!("Respuesta PVGIS no válida: {e}")))?;
    let mut anual: BTreeMap<i64, f64> = BTreeMap::new();
    for m in cuerpo["outputs"]["monthly"]
        .as_array()
        .ok_or_else(|| Error::Format("PVGIS sin outputs.monthly".into()))?
    {
        if let (Some(y), Some(h)) = (m["year"].as_i64(), m["H(h)_m"].as_f64()) {
            *anual.entry(y).or_insert(0.0) += h;
        }
    }
    let sumas: Vec<f64> = anual.values().copied().collect();
    coef_variacion(&sumas)
        .map(|cv| cv * 100.0)
        .ok_or_else(|| Error::Format("PVGIS con menos de 2 años completos".into()))
}

pub(crate) fn sigma_meteo(c: &Caso, args: &Value) -> Result<(f64, String, Option<String>)> {
    if args.get(SIGMA_METEO.clave).is_some() {
        return Ok((acotado(args, &SIGMA_METEO, None)?, "usuario".into(), None));
    }
    if !flag(args, &["internet"]) {
        return Ok((SIGMA_METEO_DEFECTO, "defecto".into(), None));
    }
    Ok(match sigma_meteo_pvgis(c) {
        Ok(s) => (s, "PVGIS MRcalc".into(), None),
        Err(e) => (SIGMA_METEO_DEFECTO, "defecto".into(), Some(e.to_string())),
    })
}

pub(crate) fn sigma_modelo(args: &Value) -> Result<f64> {
    acotado(args, &SIGMA_MODELO, Some(SIGMA_MODELO_DEFECTO))
}

pub(crate) fn incertidumbre(p50: f64, sigma_meteo: f64, sigma_modelo: f64, fuente: &str) -> Value {
    let sigma = (sigma_meteo * sigma_meteo + sigma_modelo * sigma_modelo).sqrt() / 100.0;
    json!({
        "p50_kwh": round1(p50),
        "p90_kwh": round1(p50 * (1.0 - Z_P90 * sigma)),
        "p99_kwh": round1(p50 * (1.0 - Z_P99 * sigma)),
        "sigma_total_pct": round1(sigma * 100.0),
        "sigma_meteo_pct": sigma_meteo,
        "sigma_modelo_pct": sigma_modelo,
        "sigma_meteo_fuente": fuente,
    })
}

pub(crate) fn p50_p90(paths: &Paths, args: &Value) -> Result<Value> {
    let c = caso(paths, args)?;
    let (sm, fuente, aviso) = sigma_meteo(&c, args)?;
    let mut v = incertidumbre(c.energia_kwh, sm, sigma_modelo(args)?, &fuente);
    if let Some(a) = aviso {
        v["aviso"] = json!(a);
    }
    Ok(con_cabecera(&c, v))
}
