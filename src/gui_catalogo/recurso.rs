//! `pvsyst://gui`: the catalogue plus the window map observed on a live PVsyst.

use super::dominio::Funcion;
use super::entradas::funciones;
use serde_json::{json, Value};

/// Windows seen while walking the interface: classes, titles, tabs and controls.
const MAPA_DE_VENTANAS: &str = include_str!("../../docs/gui_mapa.json");

/// Machine-readable map of the GUI: every catalogued function and every window.
pub(crate) fn como_recurso() -> Value {
    let mapa: Value = serde_json::from_str(MAPA_DE_VENTANAS).unwrap_or(Value::Null);
    json!({
        "uso": "gui_actuar accion:funcion funcion:<clave> abre una función; gui_actuar accion:menu ventana id abre un menú no catalogado.",
        "funciones": funciones().map(Funcion::a_json).collect::<Vec<_>>(),
        "mapa": mapa,
    })
}
