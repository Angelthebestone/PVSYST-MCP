//! `vista.campos`: the relevant values of an answer, with stable keys.

use super::tipos::{Campo, Recorte};
use crate::args::flag;
use serde_json::{Map, Value};
use std::collections::HashSet;

/// Most keyed values `vista.campos` lists; the rest stay in the answer itself.
const MAX_CAMPOS: usize = 12;
/// A nested object with more scalars than this is a table, not a group of values.
const MAX_HIJOS: usize = 8;
/// Longer texts (paths, logs) are not "values"; they stay in the answer.
const MAX_CARACTERES_VALOR: usize = 40;

/// Keys that carry warnings, as strings or lists of strings. They are not values
/// of the result either, so `CLAVES_AJENAS` does not repeat them.
pub(super) const CLAVES_AVISO: &[&str] = &[
    "avisos",
    "advertencias",
    "warnings",
    "aviso",
    "aviso_origen",
];

/// Bookkeeping and bulky keys that are not values of the result.
const CLAVES_AJENAS: &[&str] = &[
    "elapsed_ms",
    "siguiente",
    "hint",
    "truncado",
    "total",
    "ignorados",
    "origen",
    "errores",
    "ok",
    "vista",
    "text",
    "contenido",
    "base64",
    "_imagen_png_base64",
];

/// Unit by key suffix, the convention of the form schemas (`potencia_nominal_w`).
/// Longest suffix first: the first match wins.
const UNIDADES: &[(&str, &str)] = &[
    ("_kwh_kwp", "kWh/kWp"),
    ("_kwh_m2", "kWh/m²"),
    ("_wh_m2", "Wh/m²"),
    ("_kwh", "kWh"),
    ("_mwh", "MWh"),
    ("_kwp", "kWp"),
    ("_kw", "kW"),
    ("_tco2", "tCO2"),
    ("_bytes", "B"),
    ("_pct", "%"),
    ("_deg", "°"),
    ("_eur", "EUR"),
    ("_ms", "ms"),
    ("_m2", "m²"),
    ("_kg", "kg"),
    ("_w", "W"),
    ("_v", "V"),
    ("_a", "A"),
    ("_c", "°C"),
];

/// Unit implied by the last segment of `clave`.
pub(super) fn unidad_de(clave: &str) -> Option<String> {
    let ultimo = clave.rsplit('.').next().unwrap_or(clave);
    UNIDADES
        .iter()
        .find(|(sufijo, _)| ultimo.ends_with(sufijo))
        .map(|(_, unidad)| (*unidad).to_string())
}

fn es_valor(clave: &str, valor: &Value) -> bool {
    if CLAVES_AJENAS.contains(&clave) || CLAVES_AVISO.contains(&clave) || clave.ends_with("_total")
    {
        return false;
    }
    match valor {
        Value::String(s) => s.chars().count() <= MAX_CARACTERES_VALOR,
        Value::Number(_) | Value::Bool(_) => true,
        _ => false,
    }
}

/// Scalar values of `mapa`, one level into small nested objects (`site.annual_ghi_kwh_m2`).
fn escalares(mapa: &Map<String, Value>, prefijo: &str) -> Vec<(String, Value)> {
    mapa.iter()
        .flat_map(|(k, v)| {
            let clave = format!("{prefijo}{k}");
            match v {
                Value::Object(hijo)
                    if prefijo.is_empty() && !CLAVES_AJENAS.contains(&k.as_str()) =>
                {
                    grupo(escalares(hijo, &format!("{clave}.")))
                }
                _ if es_valor(k, v) => vec![(clave, v.clone())],
                _ => Vec::new(),
            }
        })
        .collect()
}

/// The scalars of a nested object, or none when it is too big to be a group of values.
fn grupo(hijos: Vec<(String, Value)>) -> Vec<(String, Value)> {
    if hijos.len() > MAX_HIJOS {
        return Vec::new();
    }
    hijos
}

/// A form field the GUI answer already keys: `{clave, valor, unidad, editable}`.
fn campo_de_formulario(item: &Value) -> Option<Campo> {
    let clave = item.get("clave")?.as_str()?;
    Some(Campo {
        clave: clave.to_string(),
        valor: item.get("valor").cloned().unwrap_or(Value::Null),
        unidad: item
            .get("unidad")
            .and_then(Value::as_str)
            .filter(|u| !u.is_empty())
            .map(str::to_string),
        editable: flag(item, &["editable"]),
    })
}

/// Fields of a form answer (`campos` with `clave`), or `None` when the answer is not a form.
fn campos_de_formulario(salida: &Value) -> Option<Vec<Campo>> {
    let items = salida.get("campos")?.as_array()?;
    let campos: Vec<Campo> = items.iter().filter_map(campo_de_formulario).collect();
    (!campos.is_empty()).then_some(campos)
}

/// Keyed values of `salida` and the cut, if more than [`MAX_CAMPOS`] qualify.
pub(super) fn campos_de(salida: &Value) -> (Vec<Campo>, Option<Recorte>) {
    if let Some(formulario) = campos_de_formulario(salida) {
        return acotar(formulario);
    }
    let Some(mapa) = salida.as_object() else {
        return (Vec::new(), None);
    };
    let todos: Vec<Campo> = escalares(mapa, "")
        .into_iter()
        .map(|(clave, valor)| Campo {
            unidad: unidad_de(&clave),
            clave,
            valor,
            editable: false,
        })
        .collect();
    acotar(todos)
}

/// The first [`MAX_CAMPOS`] fields and the cut when there are more: the answer's
/// own `campos` array keeps all of them.
fn acotar(mut campos: Vec<Campo>) -> (Vec<Campo>, Option<Recorte>) {
    // Two paths can flatten to one key (`{"a.b":1,"a":{"b":2}}`): the first one stays.
    let mut vistas = HashSet::new();
    campos.retain(|c| vistas.insert(c.clave.clone()));
    let total = campos.len();
    campos.truncate(MAX_CAMPOS);
    let recorte = (total > MAX_CAMPOS).then(|| Recorte {
        clave: "vista.campos".into(),
        mostrados: MAX_CAMPOS,
        total,
        pedir_resto: "leer las claves del resto en la propia respuesta",
    });
    (campos, recorte)
}
