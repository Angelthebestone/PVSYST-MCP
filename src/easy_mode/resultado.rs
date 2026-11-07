//! The design answer: the run's steps, warnings and headline figures as JSON.

use super::parametros::DisenoParametros;
use super::pasos::Corrida;
use serde_json::{json, Value};

/// Warning when the realised kWp is more than 5 % away from the target.
const MAX_DEVIATION_PCT: f64 = 5.0;

/// The whole design run as the caller sees it.
pub(super) fn serializar_resultado(params: &DisenoParametros, c: &Corrida) -> Value {
    let pasos_json = c.a_pasos_json();
    let kpis_error_str = c.simulacion.as_ref().and_then(|s| s.kpis_error.as_deref());
    let errores = errores_de(&pasos_json, kpis_error_str);
    let mut avisos = avisos_de(&pasos_json);
    let kwp_realizado_val = c
        .dimensionado
        .kwp_realizado
        .map_or(Value::Null, |k| json!(k));
    if let Some(a) = aviso_desviacion(params.dimensionado.kwp, &kwp_realizado_val) {
        avisos.push(a);
    }
    if let Some(s) = &c.simulacion {
        avisos.extend(s.avisos_pvsyst.iter().cloned());
    }

    let site_info = json!({
        "name": c.sitio.site_name,
        "file": c.sitio.sit_file,
        "weather_source": c.sitio.weather_source,
        "annual_ghi_kwh_m2": c.sitio.annual_ghi_kwh_m2,
        "reutilizado": c.sitio.reutilizado,
    });
    let sim_info = c
        .simulacion
        .as_ref()
        .map_or(Value::Null, |s| s.sim_info.clone());
    let kpis_val = c
        .simulacion
        .as_ref()
        .and_then(|s| s.kpis.as_ref())
        .map_or(Value::Null, |k| json!(k));

    json!({
        "ok": errores.is_empty(),
        "errores": errores,
        "avisos": avisos,
        "proyecto": params.proyecto.nombre,
        "tipo_sistema": params.tipo_sistema.nombre(),
        "variante": params.variante,
        "ubicacion": { "latitud": params.ubicacion.latitud, "longitud": params.ubicacion.longitud },
        "kwp_objetivo": params.dimensionado.kwp,
        "kwp_realizado": kwp_realizado_val,
        "site": site_info,
        "orientacion": { "tilt_deg": c.orientacion.tilt, "azimut_deg": c.orientacion.azimut },
        "proyecto_creado": c.proyecto.raw_response.get("project"),
        "simulacion": sim_info,
        "kpis": kpis_val,
        "pasos": pasos_json,
    })
}

/// Failed steps and a failed KPI read, as human-readable messages.
fn errores_de(pasos: &[Value], kpis_error: Option<&str>) -> Vec<String> {
    let mut errores: Vec<String> = pasos
        .iter()
        .filter(|p| p.get("ok").and_then(Value::as_bool) != Some(true))
        .map(|p| {
            let paso = p.get("paso").and_then(Value::as_str).unwrap_or("?");
            match p.get("detail").and_then(Value::as_str) {
                Some(d) => format!("paso '{paso}' fallido: {d}"),
                None => format!("paso '{paso}' fallido"),
            }
        })
        .collect();
    if let Some(e) = kpis_error {
        errores.push(format!("leer_kpis: {e}"));
    }
    errores
}

/// Warnings raised by the steps (`pasos[].avisos`), flattened.
fn avisos_de(pasos: &[Value]) -> Vec<String> {
    pasos
        .iter()
        .filter_map(|p| p.get("avisos")?.as_array())
        .flatten()
        .filter_map(Value::as_str)
        .map(str::to_string)
        .collect()
}

/// Warning when the realised kWp is more than 5 % away from the target.
fn aviso_desviacion(objetivo: f64, realizado: &Value) -> Option<String> {
    let real = realizado.as_f64()?;
    let pct = desviacion_pct(objetivo, real);
    (pct.abs() > MAX_DEVIATION_PCT).then(|| {
        format!("kWp realizado {real:.1} se desvía {pct:+.1} % del objetivo {objetivo:.1}")
    })
}

/// Percentage by which `real` departs from `objetivo`.
pub(super) fn desviacion_pct(objetivo: f64, real: f64) -> f64 {
    (real - objetivo) / objetivo * 100.0
}
