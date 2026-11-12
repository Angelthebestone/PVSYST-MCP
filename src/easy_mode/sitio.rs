//! Step 1: Geographic site creation or cache reuse from coordinates.

use super::parametros::UbicacionParametros;
use super::pasos::{OrigenSitioDiseno, PasoSitioSalida};
use crate::config::Paths;
use crate::error::{Error, Result};
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
        detalle_cli: detalle_cli(&site_info, ubicacion),
    })
}

/// `create_site` envelope plus the coordinates the CLI was asked for.
fn detalle_cli(site_info: &Value, ubicacion: &UbicacionParametros) -> Value {
    let mut detalle = site_info.get("create_site").cloned().unwrap_or(Value::Null);
    if let Some(obj) = detalle.as_object_mut() {
        obj.insert("latitud".into(), json!(ubicacion.latitud));
        obj.insert("longitud".into(), json!(ubicacion.longitud));
    }
    detalle
}

/// A site step that created nothing stops the run before a project is linked to it.
/// The error carries the CLI envelope so the caller sees why `create-site` failed.
pub(crate) fn exigir_sitio_creado(paso: &PasoSitioSalida) -> Result<()> {
    if paso.ok && !paso.sit_file.is_empty() {
        return Ok(());
    }
    Err(Error::Cli(mensaje_sitio_fallido(paso)))
}

fn mensaje_sitio_fallido(paso: &PasoSitioSalida) -> String {
    let detalle = &paso.detalle_cli;
    let site = paso.site_name.as_deref().unwrap_or("?");
    let clasificacion = detalle
        .get("clasificacion")
        .and_then(Value::as_str)
        .unwrap_or("desconocida");
    let causa = detalle
        .get("crash")
        .or_else(|| detalle.get("detected_error"))
        .or_else(|| detalle.get("motivo_ambiguedad"))
        .and_then(Value::as_str)
        .unwrap_or("sin detalle del CLI");
    let expected_sit = detalle
        .get("expected_sit")
        .and_then(Value::as_str)
        .filter(|s| !s.is_empty())
        .unwrap_or(&paso.sit_file);
    let expected_sit = if expected_sit.is_empty() {
        "?"
    } else {
        expected_sit
    };
    let exit_code = detalle
        .get("exit_code")
        .map_or_else(|| "?".to_string(), Value::to_string);
    let weather = paso
        .weather_source
        .as_deref()
        .unwrap_or("sin fuente de meteo");
    let stdout = resumen_salida(detalle, "stdout_tail");
    let stdout = if stdout.is_empty() {
        "vacío".to_string()
    } else {
        stdout
    };
    let stderr = resumen_salida(detalle, "stderr_tail");
    let coords = coords(detalle);
    let mut mensaje = format!(
        "PVsystCLI no creó el sitio '{site}' (create-site clasificacion={clasificacion}, causa={causa}, exit_code={exit_code}); esperado en '{expected_sit}' para {coords} ({weather}). stdout: {stdout}."
    );
    if !stderr.is_empty() {
        mensaje.push_str(&format!(" stderr: {stderr}."));
    }
    mensaje.push_str(
        " Revisa la salida de 'sitio crear' con las mismas coordenadas antes de crear el proyecto.",
    );
    mensaje
}

/// `lat=…, lon=…` from the envelope, or a readable fallback.
fn coords(detalle: &Value) -> String {
    match (
        detalle.get("latitud").and_then(Value::as_f64),
        detalle.get("longitud").and_then(Value::as_f64),
    ) {
        (Some(lat), Some(lon)) => format!("lat={lat}, lon={lon}"),
        _ => "coordenadas desconocidas".to_string(),
    }
}

/// Last non-empty lines of a CLI tail, capped so the error stays readable.
fn resumen_salida(detalle: &Value, clave: &str) -> String {
    let texto = detalle
        .get(clave)
        .and_then(Value::as_str)
        .unwrap_or("")
        .trim();
    if texto.is_empty() {
        return String::new();
    }
    let lineas: Vec<&str> = texto.lines().filter(|l| !l.trim().is_empty()).collect();
    let resumen = lineas
        .iter()
        .rev()
        .take(3)
        .rev()
        .copied()
        .collect::<Vec<_>>()
        .join(" | ");
    if resumen.chars().count() > 300 {
        let corto: String = resumen.chars().take(300).collect();
        format!("{corto}…")
    } else {
        resumen
    }
}
