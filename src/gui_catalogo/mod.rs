//! Catalogue of PVsyst GUI functions: which menu item or button opens each one,
//! with the window it opens. The domain (`dominio`) is pure data and lookup;
//! `aplicacion` drives the live PVsyst through the GUI adapter.

mod aplicacion;
mod dominio;
mod entradas;
mod espera;
mod recurso;

pub(crate) use aplicacion::{abrir, listar, menu};
pub(crate) use recurso::{como_recurso, mapa_de_ventanas};

/// Keys of the catalogue functions that open window class `clase`, in catalogue
/// order. O(n) in the number of catalogued functions.
pub(crate) fn funciones_que_abren(clase: &str) -> Vec<&'static str> {
    entradas::funciones()
        .filter(|f| f.abre == Some(clase))
        .map(|f| f.clave)
        .collect()
}

#[cfg(test)]
#[path = "../../tests/unit/gui_catalogo/mod.rs"]
mod tests;
