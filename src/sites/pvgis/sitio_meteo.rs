//! The site a PVGIS weather request works on: the SIT the project will use and
//! the time zone of its hours, created from the request coordinates when it
//! does not exist yet.

use std::path::{Path, PathBuf};

use serde_json::{json, Value};

use crate::args::opt_str;
use crate::config::Paths;
use crate::error::{Error, Result};
use crate::format::object;
use crate::sites;
use crate::workspace::{resolver_ruta, Alcance};

use super::persistencia;

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

/// Context bundle for weather generation workflow.
pub(super) struct ContextoMeteo<'a> {
    pub(super) paths: &'a Paths,
    pub(super) args: &'a Value,
    pub(super) ws: &'a Path,
    pub(super) nombre: &'a str,
    pub(super) lat: f64,
    pub(super) lon: f64,
}

/// Resolves the site file (or creates it) and extracts timezone.
pub(super) fn asegurar_sitio(ctx: &ContextoMeteo<'_>) -> Result<(PathBuf, i64)> {
    let alcance = Alcance::de_args(ctx.args);
    let sit_arg = opt_str(ctx.args, &["sit_file"]);
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
