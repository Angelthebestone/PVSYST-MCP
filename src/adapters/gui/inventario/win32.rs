//! Reads the windows of a form from Win32: every descendant, the hidden ones
//! included. `EnumChildWindows` lists them all, so no tab has to be switched
//! to see what is on it (switching can trip PVsyst's validation).

use super::tipos::{Instantanea, NodoWin32, Rol};
use crate::adapters::gui::ffi::{self, Hwnd, GA_PARENT};
use crate::adapters::gui::ocr::esta_marcado;
use crate::adapters::gui::ventanas::{class_of, rect_of};

/// Windows read from one form. A PVsyst form has a few hundred; the cap keeps a
/// runaway enumeration bounded.
const MAX_NODOS: usize = 2000;

fn nodo_de(c: Hwnd) -> NodoWin32 {
    let clase = class_of(c);
    let estado = ffi::estado_ventana(c);
    let marcado = if matches!(Rol::de_clase(&clase), Rol::Casilla | Rol::Opcion) {
        esta_marcado(c)
    } else {
        None
    };
    NodoWin32 {
        hwnd: c,
        padre: ffi::get_ancestor(c, GA_PARENT),
        texto: ffi::texto_control(c),
        rect: rect_of(c),
        visible: estado.visible,
        habilitado: estado.habilitada,
        marcado,
        clase,
    }
}

/// The descendants of `h`; only the visible ones unless `todos`. O(n) messages.
pub(crate) fn nodos_de(h: Hwnd, todos: bool) -> Vec<NodoWin32> {
    ffi::enum_children(h)
        .into_iter()
        .take(MAX_NODOS)
        .map(nodo_de)
        .filter(|n| todos || n.visible)
        .collect()
}

/// Page the form's first page control reports as current.
pub(crate) fn pagina_actual(nodos: &[NodoWin32]) -> Option<usize> {
    nodos
        .iter()
        .find(|n| n.clase == ffi::CLASE_PESTANAS)
        .and_then(|n| ffi::pagina_seleccionada(n.hwnd))
}

/// The form `h` as the domain reads it. `pestanas_uia` is what UI Automation
/// lists for the tab strip (empty when it could not be read).
pub(crate) fn instantanea_de(h: Hwnd, todos: bool, pestanas_uia: Vec<String>) -> Instantanea {
    instantanea_con(h, nodos_de(h, todos), pestanas_uia)
}

/// The form `h` with the windows already read.
pub(crate) fn instantanea_con(
    h: Hwnd,
    nodos: Vec<NodoWin32>,
    pestanas_uia: Vec<String>,
) -> Instantanea {
    Instantanea {
        seleccionada: pagina_actual(&nodos),
        nodos,
        origen: rect_of(h),
        escala: ffi::escala_de_captura(h),
        pestanas_uia,
    }
}

#[cfg(test)]
#[path = "../../../../tests/unit/adapters/gui/inventario/win32.rs"]
mod tests;
