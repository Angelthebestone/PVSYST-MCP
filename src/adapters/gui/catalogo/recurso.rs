//! `pvsyst://gui`: the catalogue plus the window map observed on a live PVsyst.

use super::dominio::Funcion;
use super::entradas::funciones;
use crate::idioma::{Idioma, Texto};
use serde_json::{json, Value};
use std::sync::OnceLock;

/// Windows seen while walking the interface: classes, titles, tabs and controls.
const MAPA_DE_VENTANAS: &str = include_str!("../../../../docs/gui_mapa.json");

/// The window map, parsed once; `Null` if the embedded file is not valid JSON.
pub(crate) fn mapa_de_ventanas() -> &'static Value {
    static MAPA: OnceLock<Value> = OnceLock::new();
    MAPA.get_or_init(|| serde_json::from_str(MAPA_DE_VENTANAS).unwrap_or(Value::Null))
}

/// Machine-readable map of the GUI: every catalogued function and every window.
pub(crate) fn como_recurso() -> Value {
    let mapa = mapa_de_ventanas();
    // `server::resources` reads this resource without a `Paths`: the environment
    // is the same source `Paths` reads.
    let uso = Texto {
        es: "gui_actuar accion:funcion funcion:<clave> abre una función; gui_actuar accion:menu ventana id abre un menú no catalogado.",
        en: "gui_actuar accion:funcion funcion:<clave> opens a function; gui_actuar accion:menu ventana id opens a menu that is not catalogued.",
    };
    json!({
        "uso": uso.en_idioma(Idioma::de_entorno()),
        "funciones": funciones().map(Funcion::a_json).collect::<Vec<_>>(),
        "mapa": mapa,
    })
}
