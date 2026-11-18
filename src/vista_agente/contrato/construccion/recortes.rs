//! What the detail level left out: every list the answer trimmed carries its
//! `<clave>_total` next to it, at whatever depth the trimmer cut it.

use super::super::tipos::Recorte;
use serde_json::{Map, Value};

const PEDIR_RESTO: &str = "detalle:completo";
const SUFIJO_TOTAL: &str = "_total";

/// Lists the answer cut by the detail level, from the `<clave>_total` the trimmer adds.
pub(super) fn recortes_de(salida: &Value) -> Vec<Recorte> {
    let mut recortes = Vec::new();
    recoger(salida, "", &mut recortes);
    recortes
}

/// Collects the cuts of `valor` and of everything under it. The trimmer also cuts
/// lists nested in an object (a project's variants), so the cut is named by where
/// it lives: `projects[0].variants`.
fn recoger(valor: &Value, ruta: &str, recortes: &mut Vec<Recorte>) {
    match valor {
        Value::Object(mapa) => {
            recortes.extend(recortes_del_objeto(mapa, ruta));
            for (clave, hijo) in mapa.iter() {
                recoger(hijo, &ruta_de(ruta, clave), recortes);
            }
        }
        Value::Array(items) => {
            for (i, item) in items.iter().enumerate() {
                recoger(item, &format!("{ruta}[{i}]"), recortes);
            }
        }
        _ => {}
    }
}

/// Path of a value: `projects` at the top level, `projects[0].variants` below it.
fn ruta_de(ruta: &str, clave: &str) -> String {
    if ruta.is_empty() {
        clave.to_string()
    } else {
        format!("{ruta}.{clave}")
    }
}

/// The `<clave>_total` keys of this level, each paired with the list it counted.
fn recortes_del_objeto(mapa: &Map<String, Value>, ruta: &str) -> Vec<Recorte> {
    mapa.iter()
        .filter_map(|(k, total)| Some((k.strip_suffix(SUFIJO_TOTAL)?, total.as_u64()?)))
        .filter_map(|(base, total)| {
            let mostrados = mapa.get(base)?.as_array()?.len();
            let total = usize::try_from(total).ok()?;
            // Nothing was cut when the list is as long as its total.
            (mostrados < total).then(|| Recorte {
                clave: ruta_de(ruta, base),
                mostrados,
                total,
                pedir_resto: PEDIR_RESTO,
            })
        })
        .collect()
}
