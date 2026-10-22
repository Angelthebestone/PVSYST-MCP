//! Catalogue of PVsyst GUI functions: which menu item or button opens each one,
//! with the window it opens. The domain (`dominio`) is pure data and lookup;
//! `aplicacion` drives the live PVsyst through the GUI adapter.

mod aplicacion;
mod dominio;
mod entradas;
mod espera;
mod recurso;

pub(crate) use aplicacion::{abrir, listar, menu};
pub(crate) use recurso::como_recurso;

#[cfg(test)]
#[path = "../../tests/unit/gui_catalogo/mod.rs"]
mod tests;
