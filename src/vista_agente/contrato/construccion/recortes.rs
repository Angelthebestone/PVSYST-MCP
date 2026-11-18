//! What the detail level left out: every list the answer trimmed carries its
//! `<clave>_total` next to it.

use super::super::tipos::Recorte;
use serde_json::Value;

const PEDIR_RESTO: &str = "detalle:completo";
const SUFIJO_TOTAL: &str = "_total";

/// Lists the answer cut by the detail level, from the `<clave>_total` the trimmer adds.
pub(super) fn recortes_de(salida: &Value) -> Vec<Recorte> {
    let Some(mapa) = salida.as_object() else {
        return Vec::new();
    };
    mapa.iter()
        .filter_map(|(k, total)| Some((k.strip_suffix(SUFIJO_TOTAL)?, total.as_u64()?)))
        .filter_map(|(base, total)| {
            let mostrados = mapa.get(base)?.as_array()?.len();
            let total = usize::try_from(total).ok()?;
            // Nothing was cut when the list is as long as its total.
            (mostrados < total).then(|| Recorte {
                clave: base.to_string(),
                mostrados,
                total,
                pedir_resto: PEDIR_RESTO,
            })
        })
        .collect()
}
