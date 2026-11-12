//! What UI Automation adds to the Win32 inventory: the tab strip in order,
//! which Win32 cannot give, and the elements a control draws itself (cells,
//! labels, items), which have no window handle.

use super::tipos::{LecturaUia, NodoUia};
use crate::adapters::gui::ffi::{Hwnd, Rect};
use crate::adapters::gui::uia_vista::{aplanar, resumen_uia};
use crate::adapters::uia;
use crate::error::Result;
use crate::state::Estado;
use serde_json::Value;

const PROFUNDIDAD: u32 = 6;
const MAX_NODOS: u32 = 400;
const ROL_PESTANA: &str = "tab item";

fn argumento(args: &Value, clave: &str, defecto: u32) -> u32 {
    args.get(clave)
        .and_then(Value::as_u64)
        .and_then(|n| u32::try_from(n).ok())
        .unwrap_or(defecto)
}

fn hwnd_de(nodo: &Value) -> Option<Hwnd> {
    let texto = nodo.get("hwnd")?.as_str()?;
    Hwnd::from_str_radix(texto.strip_prefix("0x")?, 16).ok()
}

/// The rectangle of an element, when it has an area.
fn rect_de(nodo: &Value) -> Option<Rect> {
    let lados = nodo.get("rect")?.as_array()?;
    let lado = |i: usize| i32::try_from(lados.get(i)?.as_i64()?).ok();
    let rect = Rect {
        left: lado(0)?,
        top: lado(1)?,
        right: lado(2)?,
        bottom: lado(3)?,
    };
    (rect.right > rect.left && rect.bottom > rect.top).then_some(rect)
}

fn texto_de(nodo: &Value, clave: &str) -> String {
    nodo.get(clave)
        .and_then(Value::as_str)
        .unwrap_or_default()
        .to_string()
}

fn nodo_uia(nodo: &Value) -> NodoUia {
    NodoUia {
        hwnd: hwnd_de(nodo),
        nombre: texto_de(nodo, "nombre"),
        rol: texto_de(nodo, "rol"),
        clase: texto_de(nodo, "clase"),
        rect: rect_de(nodo),
        patrones: nodo
            .get("patrones")
            .and_then(Value::as_array)
            .into_iter()
            .flatten()
            .filter_map(|p| p.as_str().map(str::to_string))
            .collect(),
        solo_vision: resumen_uia(nodo)
            .get("solo_vision")
            .and_then(Value::as_bool)
            == Some(true),
    }
}

/// The elements under the root of a UI Automation tree (the root is the form itself).
pub(super) fn lectura_de(arbol: &Value) -> LecturaUia {
    let mut plano = Vec::new();
    aplanar(arbol, &mut plano);
    let nodos: Vec<NodoUia> = plano.into_iter().map(nodo_uia).collect();
    let pestanas = nodos
        .iter()
        .filter(|n| n.rol == ROL_PESTANA)
        .map(|n| n.nombre.clone())
        .collect();
    LecturaUia { pestanas, nodos }
}

/// The UI Automation tree of the form `h`, `profundidad` levels deep and at
/// most `max_nodos` elements (both read from the call).
pub(super) fn leer(estado: &Estado, h: Hwnd, args: &Value) -> Result<LecturaUia> {
    let arbol = uia::arbol(
        estado,
        h,
        argumento(args, "profundidad", PROFUNDIDAD),
        argumento(args, "max_nodos", MAX_NODOS),
    )?;
    Ok(lectura_de(&arbol))
}

#[cfg(test)]
#[path = "../../../../tests/unit/adapters/gui/inventario/uia.rs"]
mod tests;
