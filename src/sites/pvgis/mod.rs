//! PVGIS web services integration: horizon profiles and hourly TMY weather data.
//!
//! Provides transactional meteorological downloads, isolated MEF conversion,
//! and horizon insertion into project variants.

pub(crate) mod adquisicion;
mod horizonte;
pub(crate) mod normalizacion;
pub(crate) mod persistencia;
mod sitio_meteo;
mod staging;
#[cfg(test)]
#[path = "../../../tests/unit/sites/pvgis/tests.rs"]
mod tests;

use sitio_meteo::{asegurar_sitio, ContextoMeteo};
use staging::{ConfiguracionStaging, StagingMeteo};

use std::path::{Path, PathBuf};

use serde_json::{json, Map, Value};

use crate::args::{opt_str, req_numero, req_str, workspace_of};

use crate::config::Paths;
use crate::error::Result;
use crate::workspace::{resolver_ruta, validar_nombre, Alcance};
pub(crate) use horizonte::horizonte;

/// Resolves final destination paths for MET and CSV files.
fn destinos_meteo(ws: &Path, args: &Value, nombre: &str) -> Result<(PathBuf, PathBuf)> {
    let alcance = Alcance::de_args(args);
    let final_met = match opt_str(args, &["output_met_file"]) {
        Some(s) => resolver_ruta(ws, s, alcance)?,
        None => ws.join("Meteo").join(format!("{nombre}_PVGIS.MET")),
    };
    let final_csv = match opt_str(args, &["output_csv_file"]) {
        Some(s) => resolver_ruta(ws, s, alcance)?,
        None => ws.join("Meteo").join(format!("{nombre}_PVGIS.csv")),
    };
    Ok((final_met, final_csv))
}

/// Parameters for building the final meteo response.
struct RespuestaMeteoParams<'a> {
    staging: &'a StagingMeteo,
    sit: &'a Path,
    num_horas: usize,
    tz: i64,
    contingencia: bool,
    resultado: Result<Value>,
    detalle_cli: Option<Value>,
}

fn construir_respuesta_meteo(p: RespuestaMeteoParams<'_>) -> Result<Value> {
    let base = json!({
        "horas": p.num_horas,
        "zona_horaria": p.tz,
        "csv": p.staging.final_csv_path.display().to_string(),
        "mef": p.staging.mef_path.display().to_string(),
        "sit": p.sit.display().to_string(),
        "contingencia_ncarfmask": p.contingencia,
    });
    match p.resultado {
        Ok(v) => {
            if let Err(e) = p.staging.commit() {
                p.staging.rollback();
                return Err(e);
            }
            Ok(with(
                base,
                json!({ "ok": true, "met": p.staging.final_met_path.display().to_string(), "cli": v }),
            ))
        }
        Err(e) => {
            let csv_error = p.staging.conservar_csv().err().map(|e| e.to_string());
            p.staging.rollback();
            let mut extra = json!({ "ok": false, "error": e.to_string() });
            if let Some(cli) = p.detalle_cli {
                extra = with(extra, diagnostico_cli(&cli));
                extra = with(extra, json!({ "cli": cli }));
            }
            if let Some(err) = csv_error {
                extra = with(extra, json!({ "csv_error": err }));
            }
            Ok(with(base, extra))
        }
    }
}

/// Trial executions one `meteo_pvgis` call can spend.
const EJECUCIONES_METEO_PVGIS: u32 = 2;

/// The two PVsystCLI commands of a `meteo_pvgis` call, for the confirmation gate.
const PLAN_METEO_PVGIS: [&str; 2] = ["create-site (solo si el sitio no existe)", "convert-meteo"];

/// Diagnostic fields of a `convertir_meteo` envelope, copied to the failure answer.
fn diagnostico_cli(cli: &Value) -> Value {
    const CAMPOS: [&str; 7] = [
        "clasificacion",
        "crash",
        "detected_error",
        "command_line",
        "stdout_tail",
        "stderr_tail",
        "exit_code",
    ];
    let mut campos = Map::new();
    for clave in CAMPOS {
        if let Some(v) = cli.get(clave) {
            campos.insert(clave.to_string(), v.clone());
        }
    }
    Value::Object(campos)
}

/// The workspace of the call, refused before the download when `convert-meteo` cannot load it.
fn workspace_para_convertir(paths: &Paths, args: &Value) -> Result<std::path::PathBuf> {
    let ws = workspace_of(paths, args);
    crate::prefs::workspace_configurado::exigir(paths, &ws, "convert-meteo")?;
    Ok(ws)
}

/// Hourly TMY from PVGIS → CSV + MEF → `.MET` via the CLI with transactional staging.
pub(crate) fn meteo_pvgis(paths: &Paths, args: &Value) -> Result<Value> {
    let ws = workspace_para_convertir(paths, args)?;
    let lat = req_numero(args, "latitud", "número")?;
    let lon = req_numero(args, "longitud", "número")?;
    let nombre = req_str(args, &["nombre_sitio"], "")?;
    validar_nombre(nombre)?;

    // Worst case of one call: `create-site` for a missing site plus `convert-meteo`.
    // Spending trial executions has to be the caller's decision, not a surprise.
    if let Some(aviso) = crate::adapters::ledger::requiere_confirmacion(
        &ws,
        args,
        EJECUCIONES_METEO_PVGIS,
        &PLAN_METEO_PVGIS.map(str::to_string),
    ) {
        return Ok(aviso);
    }

    let ctx = ContextoMeteo {
        paths,
        args,
        ws: &ws,
        nombre,
        lat,
        lon,
    };
    let (sit, tz) = asegurar_sitio(&ctx)?;
    let raw_tmy = adquisicion::descargar_tmy(lat, lon)?;
    let horas = normalizacion::parsear_tmy(&raw_tmy, tz)?;
    let (final_met, final_csv) = destinos_meteo(&ws, args, nombre)?;

    let cfg = ConfiguracionStaging {
        ws: &ws,
        nombre,
        final_met,
        final_csv,
    };
    let staging = StagingMeteo::preparar(&cfg)?;
    if let Err(e) = staging.inicializar_archivos(&normalizacion::formatear_csv(&horas)) {
        staging.rollback();
        return Err(e);
    }

    let persistencia::Conversion {
        resultado,
        contingencia,
        detalle_cli,
    } = persistencia::convertir_transaccional(paths, &staging, &sit, &ws);
    construir_respuesta_meteo(RespuestaMeteoParams {
        staging: &staging,
        sit: &sit,
        num_horas: horas.len(),
        tz,
        contingencia,
        resultado,
        detalle_cli,
    })
}

/// Merge `extra` keys into `base` (both JSON objects).
fn with(mut base: Value, extra: Value) -> Value {
    if let (Some(b), Value::Object(e)) = (base.as_object_mut(), extra) {
        b.extend(e);
    }
    base
}
