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

/// Fields and buttons of one page, the pair that exists together on screen.
type Conteo = (usize, usize);

fn mas_lleno(a: Conteo, b: Conteo) -> Conteo {
    if b.0 + b.1 > a.0 + a.1 {
        b
    } else {
        a
    }
}

/// Fields and buttons of one instance: per tab, its fullest page (the map repeats pages
/// and a tab-less window can have several), summed over tabs. Both numbers come from
/// the same page, never the best of each.
fn contar_instancia(instancia: &Value) -> Conteo {
    let paginas = instancia
        .get("paginas")
        .and_then(Value::as_array)
        .into_iter()
        .flatten();
    let mut por_pestana: HashMap<Option<&str>, Conteo> = HashMap::new();
    for p in paginas {
        let conteo = (longitud(p, "campos"), longitud(p, "botones"));
        let mejor = por_pestana
            .entry(p.get("pestana").and_then(Value::as_str))
            .or_insert(conteo);
        *mejor = mas_lleno(*mejor, conteo);
    }
    por_pestana
        .values()
        .fold((0, 0), |(c, b), (campos, botones)| {
            (c + campos, b + botones)
        })
}

pub(super) fn resumir(ventana: &Value) -> ResumenMapa {
    let instancias = ventana
        .get("instancias")
        .and_then(Value::as_array)
        .map_or(&[][..], Vec::as_slice);
    let (campos, botones) = instancias
        .iter()
        .map(contar_instancia)
        .fold((0, 0), mas_lleno);
    ResumenMapa {
        instancias: instancias.len(),
        campos,
        botones,
    }
}

/// What opened each instance of `clase` that the map recorded under `titulo`: classes shared by
/// several windows (`TChoixObjetPVForm` has 15) are told apart by their title. Empty when the
/// map has no instance with that title.
pub(crate) fn abierta_con(clase: &str, titulo: &str) -> Vec<String> {
    mapa_de_ventanas()
        .get("ventanas")
        .and_then(|v| v.get(clase))
        .and_then(|v| v.get("instancias"))
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
        .filter(|i| i.get("titulo").and_then(Value::as_str) == Some(titulo))
        .filter_map(|i| i.get("abierta_con").and_then(Value::as_str))
        .map(str::to_string)
        .collect()
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
