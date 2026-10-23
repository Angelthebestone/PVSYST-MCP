//! Menu popups. PVsyst draws its menus itself, but each popup is still a
//! `#32768` window whose `HMENU` Windows hands out, which exposes the command
//! id of every item without reading a single pixel.

use super::ventanas::{es_ventana, send_message_timeout, Mensaje, TIMEOUT_MENSAJE_MS};
use super::Hwnd;

/// `MN_GETHMENU`: asks a popup for its menu handle.
const MN_GETHMENU: u32 = 0x01E1;
/// `GetMenuItemID` answer for a submenu entry.
const ID_DE_SUBMENU: u32 = u32::MAX;
const MF_BYPOSITION: u32 = 0x0400;
const MF_SEPARATOR: u32 = 0x0800;
const MF_GRAYED: u32 = 0x0001;
const MF_DISABLED: u32 = 0x0002;
const MF_ERROR: u32 = u32::MAX;

#[link(name = "user32")]
extern "system" {
    fn GetMenuItemCount(menu: isize) -> i32;
    fn GetMenuItemID(menu: isize, posicion: i32) -> u32;
    fn GetMenuState(menu: isize, posicion: u32, banderas: u32) -> u32;
}

/// What a menu entry is.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum TipoItem {
    /// A command: its `WM_COMMAND` id.
    Comando(u32),
    Separador,
    /// Opens a submenu.
    Submenu,
}

/// One entry of an open menu, in order of appearance.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct ItemMenu {
    pub(crate) tipo: TipoItem,
    pub(crate) deshabilitado: bool,
}

/// Entries of the menu shown in `popup`; `None` when the popup has no menu.
pub(crate) fn items_de_popup(popup: Hwnd) -> Option<Vec<ItemMenu>> {
    if !es_ventana(popup) {
        return None;
    }
    let menu = send_message_timeout(popup, Mensaje::simple(MN_GETHMENU), TIMEOUT_MENSAJE_MS)?;
    if menu == 0 {
        return None;
    }
    // SAFETY: consulta de solo lectura sobre el HMENU que Windows devolvió.
    let total = unsafe { GetMenuItemCount(menu) };
    let total = u32::try_from(total).ok()?;
    Some((0..total).map(|i| item_en(menu, i)).collect())
}

fn item_en(menu: isize, posicion: u32) -> ItemMenu {
    // SAFETY: consultas de solo lectura sobre el HMENU; `posicion` está dentro
    // de `0..GetMenuItemCount` (Windows contesta -1 si no).
    let (id, estado) = unsafe {
        (
            GetMenuItemID(menu, posicion as i32),
            GetMenuState(menu, posicion, MF_BYPOSITION),
        )
    };
    let estado = if estado == MF_ERROR { 0 } else { estado };
    ItemMenu {
        tipo: tipo_de(id, estado),
        deshabilitado: estado & (MF_GRAYED | MF_DISABLED) != 0,
    }
}

fn tipo_de(id: u32, estado: u32) -> TipoItem {
    match id {
        ID_DE_SUBMENU => TipoItem::Submenu,
        _ if estado & MF_SEPARATOR != 0 => TipoItem::Separador,
        id => TipoItem::Comando(id),
    }
}
