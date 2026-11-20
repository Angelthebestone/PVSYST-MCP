//! Catalogue of PVsyst GUI functions: which menu item or button opens each one,
//! with the window it opens. The domain (`dominio`) is pure data and lookup;
//! `abrir` and `aplicacion` drive the live PVsyst through the GUI adapter.

mod abrir;
mod aplicacion;
mod dominio;
mod entradas;
mod espera;
mod recurso;

pub(crate) use abrir::abrir;
pub(crate) use aplicacion::{listar, menu};
pub(crate) use recurso::{como_recurso, como_recurso_de_ventana, mapa_de_ventanas, EnEjecucion};

/// Keys of the catalogue functions that open window class `clase`, in catalogue
/// order: the class a function announces and the alternative one it shows for
/// another kind of project both count, because PVsyst reaches the window through
/// either. O(n) in the number of catalogued functions.
pub(crate) fn funciones_que_abren(clase: &str) -> Vec<&'static str> {
    entradas::funciones()
        .filter(|f| f.abre == Some(clase) || f.abre_tambien == Some(clase))
        .map(|f| f.clave)
        .collect()
}

/// The catalogued function behind item `item` (or command `id`) of menu `menu` of
/// window class `clase`, as `(clave, etiqueta)`: owner-drawn menus carry no text,
/// so the catalogue is what names an entry. O(n) in the catalogued functions.
pub(crate) fn funcion_de_menu(
    clase: &str,
    (menu, item): (usize, usize),
    id: Option<u32>,
) -> Option<(&'static str, &'static str)> {
    entradas::funciones()
        .filter(|f| f.ventana == clase)
        .find(|f| match f.acceso {
            dominio::Acceso::MenuPosicion { menu: m, item: i } => {
                usize::from(m) == menu && usize::from(i) == item
            }
            dominio::Acceso::Menu { id: i } => id == Some(u32::from(i)),
            dominio::Acceso::Boton { .. } => false,
        })
        .map(|f| (f.clave, f.etiqueta))
}

#[cfg(test)]
#[path = "../../../../tests/unit/adapters/gui/catalogo/mod.rs"]
mod tests;

#[cfg(test)]
#[path = "../../../../tests/unit/adapters/gui/catalogo/mapa.rs"]
mod tests_mapa;
