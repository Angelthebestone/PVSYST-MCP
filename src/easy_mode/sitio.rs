//! Step 1: Geographic site creation or cache reuse from coordinates.

use super::parametros::UbicacionParametros;
use super::pasos::{OrigenSitioDiseno, PasoSitioSalida};
use crate::config::Paths;
use crate::error::Result;
use crate::sites;
use serde_json::{json, Map, Value};

const SITE_LOG_LEVEL: u32 = 1;

/// Executes the site creation step from geographic coordinates.
pub(crate) fn ejecutar_paso_sitio(
    paths: &Paths,
    ubicacion: &UbicacionParametros,
    nombre_proyecto: &str,
    sitio: OrigenSitioDiseno,
) -> Result<PasoSitioSalida> {
    let mut sa = Map::new();
    sa.insert("latitud".into(), json!(ubicacion.latitud));
    sa.insert("longitud".into(), json!(ubicacion.longitud));
    sa.insert(
        "nombre_sitio".into(),
        json!(format!("{nombre_proyecto}_Site")),
    );
    sa.insert("log_level".into(), json!(SITE_LOG_LEVEL));
    if let Some(alt) = ubicacion.altitud {
        sa.insert("altitud".into(), json!(alt));
    }
    if let Some(tz) = ubicacion.timezone {
        sa.insert("timezone".into(), json!(tz));
    }
    if let Some(cc) = &ubicacion.country_code {
        sa.insert("country_code".into(), json!(cc));
    }
    if let Some(reg) = &ubicacion.region {
        sa.insert("region".into(), json!(reg));
    }
    sa.insert("workspace".into(), json!(paths.workspace.to_string_lossy()));

    let site_info = sites::crear_sitio_coordenadas(paths, &Value::Object(sa))?;
    let ok = site_info
        .get("create_site")
        .and_then(|v| v.get("success"))
        .and_then(Value::as_bool)
        .unwrap_or(false);
    let sit_file = site_info
        .get("sit_file")
        .and_then(Value::as_str)
        .unwrap_or("")
        .to_string();
    let site_name = site_info
        .get("site")
        .and_then(Value::as_str)
        .map(String::from);
    let weather_source = site_info
        .get("weather_source")
        .and_then(Value::as_str)
        .map(String::from);
    let annual_ghi_kwh_m2 = site_info.get("annual_ghi_kwh_m2").and_then(Value::as_f64);

    Ok(PasoSitioSalida {
        sit_file,
        site_name,
        weather_source,
        annual_ghi_kwh_m2,
        reutilizado: matches!(sitio, OrigenSitioDiseno::Reutilizado),
        ok,
    })
}
