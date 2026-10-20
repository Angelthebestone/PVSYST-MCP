//! Transactional persistence for weather files and variant horizon modifications.
//! Staging isolation guarantees that incomplete runs or CLI failures leave no partial artifacts.

use std::fs;
use std::path::Path;

use serde_json::{json, Value};

use crate::config::Paths;
use crate::error::{Error, Result};
use crate::format::{edit, object};
use crate::history;
use crate::projects::anadir;
use crate::sites;

use super::normalizacion::{cambios_perfil, puntos_existentes};
use super::staging::StagingMeteo;

/// Bytes of the `NCarFMask` field in the bundled MEF; the value is the last byte.
const MASCARA: &[u8] = b"NCarFMask=8";

/// Clear `NCarFMask` (8 → 0) in place; same length, so the rest of the file is untouched.
pub(crate) fn mascara_cero(mef: &Path) -> Result<()> {
    let mut bytes = leer_bytes(mef)?;
    let pos = bytes
        .windows(MASCARA.len())
        .position(|w| w == MASCARA)
        .ok_or_else(|| Error::Format(format!("MEF sin 'NCarFMask=8': {}", mef.display())))?;
    bytes[pos + MASCARA.len() - 1] = b'0';
    fs::write(mef, &bytes)
        .map_err(|e| Error::io(format!("No se pudo escribir {}", mef.display()), e))
}

/// Paths bundle passed to `convertir_meteo`.
pub(crate) struct RutasConversion<'a> {
    pub(crate) ws: &'a Path,
    pub(crate) csv: &'a Path,
    pub(crate) mef: &'a Path,
    pub(crate) sit: &'a Path,
    pub(crate) met: &'a Path,
}

/// Run `convertir_meteo`; a CLI rejection (`success:false`) is returned as error.
/// The workspace travels in the request so the paths resolve where they live.
pub(crate) fn invocar_cli_conversion(paths: &Paths, r: &RutasConversion<'_>) -> Result<Value> {
    let v = sites::convertir_meteo(
        paths,
        &json!({
            "workspace": r.ws.display().to_string(),
            "input_csv_file": r.csv.display().to_string(),
            "input_mef_file": r.mef.display().to_string(),
            "input_sit_file": r.sit.display().to_string(),
            "output_met_file": r.met.display().to_string(),
        }),
    )?;
    if v.get("success").and_then(Value::as_bool) == Some(false) {
        let motivo = v
            .get("detected_error")
            .and_then(Value::as_str)
            .unwrap_or("convertir_meteo sin éxito");
        return Err(Error::Cli(motivo.to_string()));
    }
    Ok(v)
}

/// Executes transactional conversion with automatic contingency retry on `NCarFMask`.
pub(crate) fn convertir_transaccional(
    paths: &Paths,
    staging: &StagingMeteo,
    sit: &Path,
    ws: &Path,
) -> (Result<Value>, bool) {
    let rutas = RutasConversion {
        ws,
        csv: &staging.csv_path,
        mef: &staging.mef_path,
        sit,
        met: &staging.staging_met_path,
    };
    match invocar_cli_conversion(paths, &rutas) {
        Ok(v) => (Ok(v), false),
        Err(_) => {
            if let Err(e) = mascara_cero(&staging.mef_path) {
                (Err(e), false)
            } else {
                (invocar_cli_conversion(paths, &rutas), true)
            }
        }
    }
}

/// Parameters for applying a horizon profile to a project variant.
pub(crate) struct PeticionHorizonte<'a> {
    pub(crate) paths: &'a Paths,
    pub(crate) ws: &'a Path,
    pub(crate) var: &'a Path,
    pub(crate) proyecto: &'a str,
    pub(crate) variante: &'a str,
    pub(crate) lat: f64,
    pub(crate) lon: f64,
    pub(crate) fhh: f64,
    pub(crate) puntos: &'a [(f64, f64)],
}

/// Edits the variant's `pvHorizon` and records an atomic history snapshot.
pub(crate) fn aplicar_horizonte(req: &PeticionHorizonte<'_>) -> Result<String> {
    let mut bytes = leer_bytes(req.var)?;
    if edit::find_span(&edit::spans_of(&bytes), "pvHorizon").is_none() {
        // The insert must resolve the project in the same workspace as the edit.
        let mut pedido = json!({
            "proyecto": req.proyecto,
            "variante": req.variante,
            "elemento": "horizonte",
        });
        pedido["workspace"] = json!(req.ws.to_string_lossy());
        anadir::anadir(req.paths, &pedido)?;
        bytes = leer_bytes(req.var)?;
    }
    let existentes = puntos_existentes(&bytes);
    let comentario = format!("Horizon from PVGIS {},{}", req.lat, req.lon);
    let cabecera = vec![
        ("Comment".to_string(), Some(comentario)),
        ("FHHDiffus".to_string(), Some(format!("{:.4}", req.fhh))),
    ];
    let (paso1, _) = edit::edit_fields(&bytes, "pvHorizon", &cabecera)?;
    let (nuevo, _) = edit::edit_fields(
        &paso1,
        "pvHorizon",
        &cambios_perfil(req.puntos, &existentes),
    )?;
    let var_buf = req.var.to_path_buf();
    let id = history::snapshot(req.ws, std::slice::from_ref(&var_buf), "horizonte_pvgis")?;
    history::escribir_atomico(req.var, &nuevo)?;
    Ok(id)
}

pub(crate) use crate::workspace::leer_bytes;

pub(crate) fn leer_texto(p: &Path) -> Result<String> {
    leer_bytes(p).map(|b| String::from_utf8_lossy(&b).into_owned())
}

/// Latitude and longitude of the project's `SitePrj` object.
pub(crate) fn sitio_prj(prj: &Path) -> Result<(f64, f64)> {
    let texto = leer_texto(prj)?;
    let raiz =
        object::parse_first(&texto).ok_or_else(|| Error::Format("PRJ sin objeto raíz".into()))?;
    let sitio = raiz
        .children
        .iter()
        .find(|c| c.role.as_deref() == Some("SitePrj"))
        .ok_or_else(|| Error::Format("PRJ sin SitePrj".into()))?;
    let campo = |k: &str| {
        sitio
            .field(k)
            .and_then(|v| v.trim().parse::<f64>().ok())
            .ok_or_else(|| Error::Format(format!("SitePrj.{k} no numérico")))
    };
    Ok((campo("Latitude")?, campo("Longitude")?))
}
