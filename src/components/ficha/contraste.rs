//! What PVsyst saved against what the datasheet asked for: a component saved with
//! other values (rounded beyond the tolerance, clamped, ignored) is reported, not
//! passed off as the datasheet.

use crate::format::numero_json;
use crate::format::object::PvNode;
use serde_json::{json, Map, Value};

/// Relative tolerance between a sent and a saved number. [INFERENCIA] PVsyst rewrites
/// values with its own number of decimals; 0.1 % covers that rounding and still flags
/// a value that was clamped or not taken.
const TOLERANCIA_RELATIVA: f64 = 1e-3;

fn coincide(enviado: &Value, guardado: &str) -> bool {
    let guardado_json = json!(guardado);
    match (numero_json(enviado), numero_json(&guardado_json)) {
        (Some(e), Some(g)) => (e - g).abs() <= TOLERANCIA_RELATIVA * e.abs().max(1.0),
        _ => enviado.as_str().map(str::trim) == Some(guardado.trim()),
    }
}

/// One `{clave, enviado, guardado}` per read-back key whose saved value is not the one
/// sent; keys that were not sent are not compared. O(k) lookups in the tree.
pub(super) fn diferencias(
    raiz: &PvNode,
    leidas: &[&str],
    enviados: &Map<String, Value>,
) -> Vec<Value> {
    leidas
        .iter()
        .filter_map(|k| Some((*k, enviados.get(*k)?)))
        .filter_map(|(k, enviado)| {
            let guardado = raiz.find_deep(k);
            let igual = guardado.is_some_and(|g| coincide(enviado, g));
            (!igual).then(|| json!({ "clave": k, "enviado": enviado, "guardado": guardado }))
        })
        .collect()
}

#[cfg(test)]
#[path = "../../../tests/unit/components/ficha/contraste.rs"]
mod tests;
