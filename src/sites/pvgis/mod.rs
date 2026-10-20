//! PVGIS web services integration: horizon profiles and hourly TMY weather data.
//!
//! Provides transactional meteorological downloads, isolated MEF conversion,
//! and horizon insertion into project variants.

pub(crate) mod adquisicion;
pub(crate) mod normalizacion;
pub(crate) mod persistencia;
mod staging;
#[cfg(test)]
#[path = "../../../tests/unit/sites/pvgis/tests.rs"]
mod tests;

use staging::{ConfiguracionStaging, StagingMeteo};

use std::path::{Path, PathBuf};

use serde_json::{json, Value};

use crate::args::{missing, opt_str, req_str, workspace_of};
use crate::config::Paths;
use crate::error::{Error, Result};
use crate::format::object;
use crate::projects::resolve_project_file;
use crate::sites;
use crate::workspace::{resolver_ruta, validar_nombre, Alcance};

/// Origin classification for site resolution in weather generation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum OrigenSitio {
    EspecificadoPorUsuario,
    InferidoPorDefecto,
}

/// DTO for creating a missing site required by weather generation.
pub(crate) struct PeticionSitioMeteo<'a> {
    pub(crate) paths: &'a Paths,
    pub(crate) args: &'a Value,
    pub(crate) sit: &'a Path,
    pub(crate) origen: OrigenSitio,
    pub(crate) latitud: f64,
    pub(crate) longitud: f64,
    pub(crate) zona_horaria: i64,
    pub(crate) nombre: &'a str,
}

/// Fill the variant's `pvHorizon` with the PVGIS horizon profile of the project site.
pub(crate) fn horizonte_pvgis(paths: &Paths, args: &Value) -> Result<Value> {
    let ws = workspace_of(paths, args);
    let proyecto = req_str(args, &["proyecto", "project"], "")?;
    let variante = opt_str(args, &["variante", "variant"]).unwrap_or("VC0");
    let prj = resolve_project_file(&ws, proyecto)?;
    let var = prj.with_extension(variante);
    if !var.exists() {
        return Err(Error::NotFound(format!(
            "Variante '{variante}' no encontrada: {}",
            var.display()
        )));
    }
    let (lat, lon) = persistencia::sitio_prj(&prj)?;
    let todos = adquisicion::descargar_perfil_horizonte(lat, lon)?;
    let total = todos.len();
    let puntos = normalizacion::diezmar(todos);
    let fhh = normalizacion::calcular_fhh(&puntos);

    let peticion = persistencia::PeticionHorizonte {
        paths,
        ws: &ws,
        var: &var,
        proyecto,
        variante,
        lat,
        lon,
        fhh,
        puntos: &puntos,
    };
    let id = persistencia::aplicar_horizonte(&peticion)?;

    Ok(json!({
        "ok": true,
        "variante": var.display().to_string(),
        "puntos": puntos.len(),
        "puntos_pvgis": total,
        "FHHDiffus": fhh,
        "factores_aproximados": true,
        "historial_id": id,
    }))
}
/// Context bundle for weather generation workflow.
struct ContextoMeteo<'a> {
    paths: &'a Paths,
    args: &'a Value,
    ws: &'a Path,
    nombre: &'a str,
    lat: f64,
    lon: f64,
}

/// Resolves the site file (or creates it) and extracts timezone.
fn asegurar_sitio(ctx: &ContextoMeteo<'_>) -> Result<(PathBuf, i64)> {
    let alcance = Alcance::de_args(ctx.args);
    let sit_arg = opt_str(ctx.args, &["sit_file", "sit"]);
    let (mut sit, origen) = match sit_arg {
        Some(s) => (
            resolver_ruta(ctx.ws, s, alcance)?,
            OrigenSitio::EspecificadoPorUsuario,
        ),
        None => (
            ctx.ws.join("Sites").join(format!("{}.SIT", ctx.nombre)),
            OrigenSitio::InferidoPorDefecto,
        ),
    };
    let tz = zona_horaria(ctx.args, &sit, ctx.lon)?;
    if !sit.exists() {
        let peticion_sitio = PeticionSitioMeteo {
            paths: ctx.paths,
            args: ctx.args,
            sit: &sit,
            origen,
            latitud: ctx.lat,
            longitud: ctx.lon,
            zona_horaria: tz,
            nombre: ctx.nombre,
        };
        sit = crear_sitio_para_meteo(&peticion_sitio)?;
    }
    Ok((sit, tz))
}

/// Resolves final destination paths for MET and CSV files.
fn destinos_meteo(ws: &Path, args: &Value, nombre: &str) -> Result<(PathBuf, PathBuf)> {
    let alcance = Alcance::de_args(args);
    let final_met = match opt_str(args, &["output_met_file", "met_file", "met"]) {
        Some(s) => resolver_ruta(ws, s, alcance)?,
        None => ws.join("Meteo").join(format!("{nombre}_PVGIS.MET")),
    };
    let final_csv = match opt_str(args, &["output_csv_file", "csv_file"]) {
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
            p.staging.commit()?;
            Ok(with(
                base,
                json!({ "ok": true, "met": p.staging.final_met_path.display().to_string(), "cli": v }),
            ))
        }
        Err(e) => {
            p.staging.rollback();
            Ok(with(base, json!({ "ok": false, "error": e.to_string() })))
        }
    }
}

/// Hourly TMY from PVGIS → CSV + MEF → `.MET` via the CLI with transactional staging.
pub(crate) fn meteo_pvgis(paths: &Paths, args: &Value) -> Result<Value> {
    let ws = workspace_of(paths, args);
    let lat = numero(args, "latitud")?;
    let lon = numero(args, "longitud")?;
    let nombre = req_str(args, &["nombre_sitio", "nombre"], "")?;
    validar_nombre(nombre)?;

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

    let (resultado, contingencia) =
        persistencia::convertir_transaccional(paths, &staging, &sit, &ws);
    construir_respuesta_meteo(RespuestaMeteoParams {
        staging: &staging,
        sit: &sit,
        num_horas: horas.len(),
        tz,
        contingencia,
        resultado,
    })
}
/// The SIT to use when `sit` does not exist: user-specified path is an error; otherwise auto-created.
fn crear_sitio_para_meteo(req: &PeticionSitioMeteo<'_>) -> Result<PathBuf> {
    if req.origen == OrigenSitio::EspecificadoPorUsuario {
        return Err(Error::NotFound(format!(
            "SIT no encontrado: {}",
            req.sit.display()
        )));
    }
    let creado = sites::crear_sitio_coordenadas(req.paths, &pedido_de_sitio(req))?;
    if creado["create_site"]["success"] != json!(true) {
        return Err(Error::Invalid(format!(
            "No se pudo crear el sitio: {creado}"
        )));
    }
    Ok(creado["sit_file"]
        .as_str()
        .map_or_else(|| req.sit.to_path_buf(), PathBuf::from))
}

/// Request payload for `crear_sitio_coordenadas`: the created site must land in
/// the same workspace as the meteo request.
pub(super) fn pedido_de_sitio(req: &PeticionSitioMeteo<'_>) -> Value {
    let mut pedido = json!({
        "latitud": req.latitud,
        "longitud": req.longitud,
        "nombre_sitio": req.nombre,
        "timezone": req.zona_horaria,
    });
    if let Some(alt) = req.args.get("altitud") {
        pedido["altitud"] = alt.clone();
    }
    if let Some(ws) = req.args.get("workspace") {
        pedido["workspace"] = ws.clone();
    }
    pedido
}

/// Merge `extra` keys into `base` (both JSON objects).
fn with(mut base: Value, extra: Value) -> Value {
    if let (Some(b), Value::Object(e)) = (base.as_object_mut(), extra) {
        b.extend(e);
    }
    base
}

/// Time zone: explicit `timezone`, else the SIT `TimeZone`, else round(lon/15);
/// held to the same bounds as `sites::solar::sitio_de`.
pub(crate) fn zona_horaria(args: &Value, sit: &Path, lon: f64) -> Result<i64> {
    let del_sit = || {
        persistencia::leer_texto(sit)
            .ok()
            .and_then(|t| object::parse_first(&t))
            .and_then(|raiz| raiz.find_deep("TimeZone").map(str::to_string))
            .and_then(|v| v.trim().parse::<f64>().ok())
    };
    let tz = args
        .get("timezone")
        .and_then(Value::as_f64)
        .or_else(del_sit)
        .unwrap_or_else(|| (lon / 15.0).round());
    let rango = crate::sites::solar::ZONA_HORARIA_MINIMA..=crate::sites::solar::ZONA_HORARIA_MAXIMA;
    if !tz.is_finite() || !rango.contains(&tz) {
        return Err(Error::Invalid(format!(
            "Zona horaria {tz} fuera de rango [{}, {}]",
            rango.start(),
            rango.end()
        )));
    }
    // Finite and within ±14, so rounding to whole hours fits i64 exactly.
    Ok(tz.round() as i64)
}

/// Numeric argument given as a JSON number or a numeric string.
fn numero(args: &Value, clave: &str) -> Result<f64> {
    args.get(clave)
        .and_then(|v| v.as_f64().or_else(|| v.as_str()?.trim().parse().ok()))
        .ok_or_else(|| missing(clave, "número"))
}
