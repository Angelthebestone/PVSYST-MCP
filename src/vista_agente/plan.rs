//! Turns the values an agent asks for into the exact text to type, checking
//! every one first so nothing is typed when any of them is wrong. Pure.

use super::dominio::{CampoEsquema, Entidad, TipoValor};
use crate::error::{Error, Result};
use serde_json::{Map, Value};

/// A validated change: the field and the text PVsyst will receive.
#[derive(Debug)]
pub(crate) struct Cambio {
    pub(crate) campo: &'static CampoEsquema,
    pub(crate) texto: String,
}

fn numero_de(valor: &Value) -> Option<f64> {
    match valor {
        Value::Number(n) => n.as_f64(),
        Value::String(s) => s.trim().replace(',', ".").parse().ok(),
        _ => None,
    }
}

/// Text of `valor` for `campo`, or why it is not acceptable.
fn texto_de(campo: &CampoEsquema, valor: &Value) -> std::result::Result<String, String> {
    match campo.tipo {
        TipoValor::Numero { minimo, maximo } => {
            let n = numero_de(valor).filter(|n| n.is_finite());
            let n = n.ok_or("debe ser un número")?;
            validar_rango(n, minimo, maximo)?;
            Ok(n.to_string())
        }
        TipoValor::Entero { minimo, maximo } => {
            let n = numero_de(valor).filter(|n| n.fract() == 0.0);
            let n = n.ok_or("debe ser un entero")? as i64;
            validar_rango(n, minimo, maximo)?;
            Ok(n.to_string())
        }
        TipoValor::Texto | TipoValor::Lista => match valor.as_str().map(str::trim) {
            Some(t) if !t.is_empty() => Ok(t.to_string()),
            _ => Err("debe ser un texto no vacío".into()),
        },
    }
}

fn validar_rango<T: Copy + PartialOrd + std::fmt::Display>(
    n: T,
    minimo: T,
    maximo: T,
) -> std::result::Result<(), String> {
    if n < minimo || n > maximo {
        Err(format!("{n} fuera de rango [{minimo}, {maximo}]"))
    } else {
        Ok(())
    }
}

/// Validates `valores` against `entidad`. Every problem is reported at once;
/// the changes come back in schema order so the run is reproducible. O(campos).
pub(crate) fn planificar(
    entidad: &'static Entidad,
    valores: &Map<String, Value>,
) -> Result<Vec<Cambio>> {
    let mut problemas = Vec::new();
    for clave in valores.keys() {
        match entidad.campo(clave) {
            None => problemas.push(format!("'{clave}' no existe")),
            Some(c) if !c.es_editable() => problemas.push(format!("'{clave}' es de solo lectura")),
            Some(_) => {}
        }
    }
    let mut cambios = Vec::new();
    for campo in entidad
        .campos
        .iter()
        .filter(|c| valores.contains_key(c.clave))
    {
        match texto_de(campo, &valores[campo.clave]) {
            Ok(texto) => cambios.push(Cambio { campo, texto }),
            Err(motivo) => problemas.push(format!("'{}' {motivo}", campo.clave)),
        }
    }
    if problemas.is_empty() && !cambios.is_empty() {
        return Ok(cambios);
    }
    if problemas.is_empty() {
        problemas.push("no hay ningún valor".into());
    }
    Err(Error::Invalid(format!(
        "{}: {}. Claves editables: {:?}",
        entidad.clave,
        problemas.join("; "),
        entidad.claves_editables()
    )))
}

/// Whether `texto` is acceptable for `campo` (type and range).
pub(crate) fn es_valido(campo: &CampoEsquema, texto: &str) -> bool {
    texto_de(campo, &Value::from(texto)).is_ok()
}

/// Whether the value read back is the one asked for: numbers compare by value
/// (PVsyst writes `200` as `200.0`), texts as they are.
pub(crate) fn coincide(campo: &CampoEsquema, pedido: &str, leido: &str) -> bool {
    match campo.tipo {
        TipoValor::Numero { .. } | TipoValor::Entero { .. } => {
            match (
                numero_de(&Value::from(pedido)),
                numero_de(&Value::from(leido)),
            ) {
                (Some(a), Some(b)) => (a - b).abs() <= f64::EPSILON * a.abs().max(1.0) * 8.0,
                _ => false,
            }
        }
        TipoValor::Texto => pedido == leido,
        TipoValor::Lista => leido.to_lowercase().starts_with(&pedido.to_lowercase()),
    }
}
