//! Application orchestration for reading PVsyst currency tables (`Currency.dat`).

use super::codec::parse_currency;
use crate::args::workspace_of;
use crate::config::{data_ro, Paths};
use crate::error::Result;
use crate::workspace::{
    leer_bytes_acotados, resolver_archivo_con_fallback, PeticionResolucion, MAX_BYTES_MAXIMO,
};
use serde_json::{json, Value};

const NOMBRE_ARCHIVO_POR_DEFECTO: &str = "Currency.dat";

/// Reads and decodes `Currency.dat`, returning its structured exchange rates representation.
pub(crate) fn leer_divisas(paths: &Paths, args: &Value) -> Result<Value> {
    let ws = workspace_of(paths, args);
    let archivo_str = args
        .get("archivo")
        .and_then(Value::as_str)
        .unwrap_or(NOMBRE_ARCHIVO_POR_DEFECTO);

    let rutas_dataro = if archivo_str.eq_ignore_ascii_case(NOMBRE_ARCHIVO_POR_DEFECTO) {
        vec![data_ro().join(NOMBRE_ARCHIVO_POR_DEFECTO)]
    } else {
        vec![data_ro().join(archivo_str)]
    };
    let peticion = PeticionResolucion {
        nombre: archivo_str,
        subcarpeta_ws: None,
        rutas_dataro,
    };
    let ruta = resolver_archivo_con_fallback(&ws, args, &peticion)?;
    let bytes = leer_bytes_acotados(&ruta, MAX_BYTES_MAXIMO)?;
    let tabla = parse_currency(&bytes)?;

    let divisas_json: Vec<Value> = tabla
        .divisas()
        .iter()
        .map(|d| {
            json!({
                "codigo": d.codigo(),
                "nombre": d.nombre(),
                "tasa_cambio": d.tasa_cambio()
            })
        })
        .collect();

    Ok(json!({
        "archivo": ruta.file_name().map_or("Currency.dat".into(), |n| n.to_string_lossy().to_string()),
        "version_formato": tabla.version_formato(),
        "moneda_base_idx": tabla.moneda_base_idx(),
        "flag": tabla.flag(),
        "total_divisas": tabla.divisas().len(),
        "divisas": divisas_json
    }))
}
