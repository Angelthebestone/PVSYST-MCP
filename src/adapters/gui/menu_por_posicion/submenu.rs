//! A command inside a submenu of a top-level menu. The popup of the top-level menu is
//! opened with the keyboard, as for any other entry, and the id of the nested command is
//! read from the `HMENU` of the submenu: the submenu itself is never entered.

use super::{exigir_menu_valido, id_de_comando, leer_con, Posicion};
use crate::adapters::gui::ffi::{self, Hwnd};
use crate::error::{Error, Result};

/// `WM_COMMAND` id of entry `sub` of the submenu at `posicion`.
pub(crate) fn id_en_submenu(ventana: Hwnd, posicion: Posicion, sub: usize) -> Result<u16> {
    exigir_menu_valido(posicion.menu)?;
    let items = leer_con(ventana, posicion.menu, |popup| {
        ffi::items_de_submenu(popup, posicion.item)
    })?;
    let item = items.get(sub).ok_or_else(|| {
        Error::NotFound(format!(
            "El submenú {} del menú {} solo tiene {} entradas",
            posicion.item,
            posicion.menu,
            items.len()
        ))
    })?;
    id_de_comando(
        item,
        &format!(
            "{sub} del submenú {} del menú {}",
            posicion.item, posicion.menu
        ),
    )
}
