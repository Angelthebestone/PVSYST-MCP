//! Application orchestration for reading and writing PVsyst `.HOR` horizon files.

use super::codec::{encode_horizonte, parse_horizonte};
use super::dominio::{validar_puntos, Horizonte, PuntoHorizonte};
use crate::args::{missing, req_str, workspace_of};
use crate::config::{data_ro, Alcance, Paths};
use crate::error::{Error, Result};
use crate::workspace::{primer_existente_en_bases, primer_existente_en_workspace};
use serde_json::{json, Value};
use std::fs;
use std::path::{Path, PathBuf};

/// Folder PVsyst reads horizons from: a `.HOR` lives there, whatever folder the
/// caller wrote in the name.
const CARPETA_HORIZONTES: &str = "Shadings";

/// Reads and decodes a `.HOR` file, returning its structured horizon representation.
pub(crate) fn leer_horizonte(paths: &Paths, args: &Value) -> Result<Value> {
    let nombre = req_str(args, &["archivo"], "p.ej. 'DEMO_Geneve.HOR'")?;
    let ruta = resolver_archivo_horizonte(paths, args, nombre)?;
    let bytes =
        fs::read(&ruta).map_err(|e| Error::io(format!("No se pudo leer {}", ruta.display()), e))?;
    let h = parse_horizonte(&bytes)?;
    Ok(json!({
        "archivo": ruta.to_string_lossy(),
        "titulo": h.titulo,
        "version": h.version,
        "calcul": h.calcul,
        "puntos": h.puntos,
    }))
}

/// Encodes and writes a `.HOR` file into the workspace under `Shadings/`, with history tracking.
pub(crate) fn escribir_horizonte(paths: &Paths, args: &Value) -> Result<Value> {
    let archivo = req_str(args, &["archivo"], "p.ej. 'MiSitio.HOR'")?;
    let titulo = args
        .get("titulo")
        .and_then(Value::as_str)
        .unwrap_or("Horizon line");
    let arr = args
        .get("puntos")
        .and_then(Value::as_array)
        .ok_or_else(|| missing("puntos", "lista de puntos [{azimut, altura}]"))?;
    let puntos = parsear_puntos_json(arr)?;
    validar_puntos(&puntos)?;

    let h = Horizonte {
        titulo: titulo.to_string(),
        version: "3.0".to_string(),
        calcul: "$0001".to_string(),
        puntos,
    };
    let bytes = encode_horizonte(&h)?;
    let destino = resolver_destino_escritura(paths, args, archivo)?;
    if let Some(parent) = destino.parent() {
        fs::create_dir_all(parent)
            .map_err(|e| Error::io(format!("No se pudo crear {}", parent.display()), e))?;
    }
    let ws = workspace_of(paths, args);
    let historial_id = crate::history::reemplazar(&ws, &destino, &bytes, "escribir_horizonte")?;
    Ok(json!({
        "archivo": destino.to_string_lossy(),
        "puntos": h.puntos.len(),
        "bytes_escritos": bytes.len(),
        "historial_id": historial_id,
    }))
}

fn resolver_archivo_horizonte(paths: &Paths, args: &Value, nombre: &str) -> Result<PathBuf> {
    let ws = workspace_of(paths, args);
    let candidatas = [
        nombre.to_string(),
        format!("{CARPETA_HORIZONTES}/{nombre}"),
        format!("{CARPETA_HORIZONTES}/{nombre}.HOR"),
        format!("{nombre}.HOR"),
    ];
    if let Some(p) = primer_existente_en_workspace(&ws, &candidatas, Alcance::de_args(args))? {
        return Ok(p);
    }
    let ro = data_ro();
    let bases = [
        ro.join("PVsyst8.1_Data").join("Shadings"),
        ro.join("Shadings"),
    ];
    let nombres = [nombre.to_string(), format!("{nombre}.HOR")];
    primer_existente_en_bases(&bases, &nombres)
        .ok_or_else(|| Error::NotFound(format!("Archivo de horizonte '{nombre}' no encontrado")))
}

/// Destination of a written horizon: `<workspace>/Shadings/<name>.HOR`.
///
/// Only the file name of `nombre` is used: `Sites/X.HOR` used to create a folder
/// `Shadings/Sites`, which is not where PVsyst looks for a horizon.
fn resolver_destino_escritura(paths: &Paths, args: &Value, nombre: &str) -> Result<PathBuf> {
    let ws = workspace_of(paths, args);
    let alcance = Alcance::de_args(args);
    let archivo = Path::new(nombre)
        .file_name()
        .and_then(|n| n.to_str())
        .ok_or_else(|| Error::Invalid(format!("'archivo' debe nombrar un archivo: '{nombre}'")))?;
    crate::workspace::validar_nombre(archivo)?;
    let con_ext = if archivo.to_ascii_uppercase().ends_with(".HOR") {
        archivo.to_string()
    } else {
        format!("{archivo}.HOR")
    };
    crate::workspace::resolver_ruta(&ws, &format!("{CARPETA_HORIZONTES}/{con_ext}"), alcance)
}

fn parsear_puntos_json(arr: &[Value]) -> Result<Vec<PuntoHorizonte>> {
    arr.iter()
        .map(|v| {
            let azimut = v
                .get("azimut")
                .or_else(|| v.get("azimut_grados"))
                .and_then(Value::as_f64)
                .ok_or_else(|| {
                    Error::Invalid("Punto de horizonte sin campo numérico 'azimut'".into())
                })?;
            let altura = v
                .get("altura")
                .or_else(|| v.get("altura_grados"))
                .and_then(Value::as_f64)
                .ok_or_else(|| {
                    Error::Invalid("Punto de horizonte sin campo numérico 'altura'".into())
                })?;
            Ok(PuntoHorizonte::de_grados(azimut, altura))
        })
        .collect()
}
