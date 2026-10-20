//! Inverter (.OND) creation from datasheet values.

use super::{aplicar, con_bom, destino, sin_bom, valor_texto};
use crate::components::buscar::numero;
use crate::config::compos_ro;
use crate::error::{Error, Result};
use crate::format::object;
use crate::history::escribir_atomico;
use serde_json::{json, Map, Value};
use std::fs;
use std::path::Path;

/// Base rated power of the generic OND template in kilowatts.
const POTENCIA_PLANTILLA_KW: f64 = 50.0;

/// OND datasheet keys grouped by the object (selector) that holds them.
const OND_CAMPOS: &[(&str, &[&str])] = &[
    ("pvCommercial", &["Manufacturer", "Model"]),
    (
        "pvGInverter",
        &[
            "PNomConv",
            "PMaxOUT",
            "VMppMin",
            "VMPPMax",
            "VAbsMax",
            "EfficMax",
            "EfficEuro",
            "IDCMax",
        ],
    ),
    ("pvGInverter", &["NbInputs", "NbMPPT"]),
];

/// The generated OND must parse back as a `pvGInverter`.
fn verificar_ond(salida: &[u8]) -> Result<()> {
    let verificado = object::parse_first(&String::from_utf8_lossy(salida))
        .ok_or_else(|| Error::Format("El OND generado no se puede interpretar".into()))?;
    if verificado.obj_type == "pvGInverter" {
        Ok(())
    } else {
        Err(Error::Invalid(format!(
            "El OND generado tiene tipo '{}' en vez de pvGInverter",
            verificado.obj_type
        )))
    }
}

/// Writes the datasheet values into the template; returns the applied fields and
/// the names of those the template keeps (not given in `datos`).
fn aplicar_campos_ond(
    bytes: &mut Vec<u8>,
    datos: &Map<String, Value>,
) -> Result<(Map<String, Value>, Vec<String>)> {
    let mut aplicados = Map::new();
    let mut heredados = Vec::new();
    for (selector, claves) in OND_CAMPOS {
        let mut cambios = Vec::new();
        for clave in claves.iter() {
            match datos.get(*clave) {
                Some(v) => {
                    let texto = valor_texto(clave, v)?;
                    aplicados.insert((*clave).to_string(), json!(texto));
                    cambios.push(((*clave).to_string(), Some(texto)));
                }
                None => heredados.push((*clave).to_string()),
            }
        }
        aplicar(bytes, selector, &cambios)?;
    }
    Ok((aplicados, heredados))
}

/// `Point_k=x,y` lines scaled by `factor` (rounded to whole watts).
pub(crate) fn puntos_escalados(texto: &str, factor: f64) -> Vec<(String, Option<String>)> {
    texto
        .lines()
        .filter_map(|linea| {
            let (clave, valor) = linea.trim().split_once('=')?;
            if !clave.starts_with("Point_") {
                return None;
            }
            let (x, y) = valor.split_once(',')?;
            let x: f64 = x.trim().parse().ok()?;
            let y: f64 = y.trim().parse().ok()?;
            Some((
                clave.trim().to_string(),
                Some(format!("{:.0},{:.0}", x * factor, y * factor)),
            ))
        })
        .collect()
}

pub(crate) fn ond(ws: &Path, nombre: &str, datos: &Map<String, Value>) -> Result<Value> {
    let plantilla = compos_ro().join("Inverters").join("Generic_50kW.OND");
    let original = fs::read(&plantilla).map_err(|e| {
        Error::io(
            format!("No se pudo leer la plantilla {}", plantilla.display()),
            e,
        )
    })?;
    let (bom, cuerpo) = sin_bom(&original);
    let mut bytes = cuerpo.to_vec();

    let (aplicados, heredados) = aplicar_campos_ond(&mut bytes, datos)?;
    let ignorados: Vec<String> = datos
        .keys()
        .filter(|k| !OND_CAMPOS.iter().any(|(_, c)| c.contains(&k.as_str())))
        .cloned()
        .collect();

    // Rescale the DC efficiency profile: its points are W in / W out for a
    // template rated at 50 kW.
    let factor = match datos.get("PNomConv").and_then(numero) {
        Some(p) if p > 0.0 => p / POTENCIA_PLANTILLA_KW,
        Some(_) => return Err(Error::Invalid("PNomConv debe ser mayor que 0".into())),
        None => 1.0,
    };
    if (factor - 1.0).abs() > f64::EPSILON {
        let texto = String::from_utf8_lossy(&bytes).into_owned();
        aplicar(&mut bytes, "pvGInverter", &puntos_escalados(&texto, factor))?;
    }

    let salida = con_bom(bom, &bytes);
    verificar_ond(&salida)?;

    let dest = destino(ws, "Inverters", &format!("{nombre}.OND"))?;
    escribir_atomico(&dest, &salida)?;
    Ok(json!({
        "ok": true,
        "tipo": "OND",
        "archivo": dest.display().to_string(),
        "plantilla": plantilla.display().to_string(),
        "aplicados": aplicados,
        "heredados": heredados,
        "ignorados": ignorados,
        "escala_puntos": factor,
    }))
}
