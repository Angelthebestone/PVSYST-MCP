//! What the observed window map says about each window class.

use crate::gui_catalogo::mapa_de_ventanas;
use serde_json::Value;
use std::collections::{BTreeMap, HashMap};
use std::sync::OnceLock;

/// Counts for one class, taken from its richest instance.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct ResumenMapa {
    pub(crate) instancias: usize,
    pub(crate) campos: usize,
    pub(crate) botones: usize,
}

fn longitud(pagina: &Value, clave: &str) -> usize {
    pagina
        .get(clave)
        .and_then(Value::as_array)
        .map_or(0, Vec::len)
}

/// Fields and buttons of one instance: per tab, its fullest page (the map repeats pages
/// and a tab-less window can have several), summed over tabs.
fn contar_instancia(instancia: &Value) -> (usize, usize) {
    let paginas = instancia
        .get("paginas")
        .and_then(Value::as_array)
        .into_iter()
        .flatten();
    let mut por_pestana: HashMap<Option<&str>, (usize, usize)> = HashMap::new();
    for p in paginas {
        let conteo = (longitud(p, "campos"), longitud(p, "botones"));
        let mejor = por_pestana
            .entry(p.get("pestana").and_then(Value::as_str))
            .or_insert(conteo);
        *mejor = (mejor.0.max(conteo.0), mejor.1.max(conteo.1));
    }
    por_pestana
        .values()
        .fold((0, 0), |(c, b), (campos, botones)| {
            (c + campos, b + botones)
        })
}

fn resumir(ventana: &Value) -> ResumenMapa {
    let instancias = ventana
        .get("instancias")
        .and_then(Value::as_array)
        .map_or(&[][..], Vec::as_slice);
    let conteos = instancias.iter().map(contar_instancia);
    ResumenMapa {
        instancias: instancias.len(),
        campos: conteos.clone().map(|c| c.0).max().unwrap_or(0),
        botones: conteos.map(|c| c.1).max().unwrap_or(0),
    }
}

/// Every window class of `docs/gui_mapa.json`, by class name. The map never
/// changes while the server runs, so it is summarised once.
pub(crate) fn clases_del_mapa() -> &'static BTreeMap<String, ResumenMapa> {
    static CLASES: OnceLock<BTreeMap<String, ResumenMapa>> = OnceLock::new();
    CLASES.get_or_init(|| {
        mapa_de_ventanas()
            .get("ventanas")
            .and_then(Value::as_object)
            .into_iter()
            .flatten()
            .map(|(clase, ventana)| (clase.clone(), resumir(ventana)))
            .collect()
    })
}
