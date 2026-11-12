//! `gui_observar que:elementos`: the rows, cells and items a control draws
//! itself (`TVirtualStringTree`, grids, lists), which have no window handle and
//! so are not in the Win32 inventory. They come from the UI Automation tree
//! under the control; a control that exposes none is vision-only, and the
//! answer says which actions still reach it.
//!
//! The walk lists the elements the control has realised. A virtualised row
//! that is scrolled out of view is not listed (its `virtualizado` and
//! `contenedor` patterns are reported, but it is not realised through them).

use super::tipos::{LecturaUia, NodoUia};
use super::{uia, TOPE_CONTROLES};
use crate::adapters::gui::ffi::{self, Hwnd, Rect};
use crate::adapters::gui::ventanas::{class_of, hwnd_de, text_of};
use crate::config::Paths;
use crate::error::Result;
use crate::idioma::{Idioma, Texto};
use serde::Serialize;
use serde_json::{json, Value};

/// One element of a control.
#[derive(Debug, Clone, PartialEq, Serialize)]
struct Elemento {
    /// Position in the order the tree lists them (top to bottom for rows).
    indice: usize,
    rol: String,
    nombre: String,
    rect: Rect,
    patrones: Vec<String>,
    /// Only when the element is itself a window.
    hwnd: Option<String>,
    solo_vision: bool,
}

fn es_elemento(n: &NodoUia) -> bool {
    n.rect.is_some() && (!n.nombre.trim().is_empty() || !n.patrones.is_empty() || n.solo_vision)
}

/// The elements of the tree that tell something, numbered in tree order.
fn elementos_de(lectura: &LecturaUia) -> Vec<Elemento> {
    lectura
        .nodos
        .iter()
        .filter(|n| es_elemento(n))
        .filter_map(|n| Some((n, n.rect?)))
        .enumerate()
        .map(|(indice, (n, rect))| Elemento {
            indice,
            rol: n.rol.clone(),
            nombre: n.nombre.clone(),
            rect,
            patrones: n.patrones.clone(),
            hwnd: n.hwnd.map(|h| format!("0x{h:X}")),
            solo_vision: n.solo_vision,
        })
        .collect()
}

const ALTERNATIVAS: Texto = Texto {
    es: "UI Automation no expone elementos en este control (solo visión). Alternativas: gui_actuar accion:fila ventana:<formulario> clase:<clase> indice:<n> (controles con geometría medida), gui_observar que:menus y gui_actuar accion:menu (menús), gui_actuar accion:clic_texto texto:<texto> (OCR).",
    en: "UI Automation exposes no elements in this control (vision only). Alternatives: gui_actuar accion:fila ventana:<form> clase:<class> indice:<n> (controls with measured geometry), gui_observar que:menus and gui_actuar accion:menu (menus), gui_actuar accion:clic_texto texto:<text> (OCR).",
};

fn respuesta(h: Hwnd, elementos: Vec<Elemento>, idioma: Idioma) -> Value {
    let total = elementos.len();
    let solo_vision = elementos.is_empty();
    json!({
        "ventana": format!("0x{h:X}"),
        "titulo": text_of(h),
        "clase": class_of(h),
        "elementos": elementos.into_iter().take(TOPE_CONTROLES).collect::<Vec<_>>(),
        "total": total,
        "truncado": total > TOPE_CONTROLES,
        "solo_vision": solo_vision,
        "alternativas": solo_vision.then(|| ALTERNATIVAS.en_idioma(idioma)),
    })
}

/// The elements of the control (or form) named by `ventana`; `profundidad` and
/// `max_nodos` bound the UI Automation tree.
pub(crate) fn elementos(paths: &Paths, args: &Value) -> Result<Value> {
    ffi::declarar_dpi_por_monitor();
    let h = hwnd_de(args, "ventana")?;
    let lectura = uia::leer(&paths.estado, h, args)?;
    Ok(respuesta(h, elementos_de(&lectura), paths.idioma))
}

#[cfg(test)]
#[path = "../../../../tests/unit/adapters/gui/inventario/elementos.rs"]
mod tests;
