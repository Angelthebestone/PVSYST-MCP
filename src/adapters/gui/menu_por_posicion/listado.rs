//! `gui_observar que:menus`: every menu of a window's bar with its entries.

use super::{es_menu_de_sistema, leer_barra, Fin, MAX_MENUS};
use crate::adapters::gui::ffi::{Hwnd, ItemMenu, TipoItem};
use crate::error::Result;
use serde_json::{json, Value};

fn item_json(posicion: usize, item: &ItemMenu) -> Value {
    let (tipo, id) = match item.tipo {
        TipoItem::Comando(id) => ("comando", Some(id)),
        TipoItem::Separador => ("separador", None),
        TipoItem::Submenu => ("submenu", None),
    };
    json!({ "posicion": posicion, "tipo": tipo, "id": id, "deshabilitado": item.deshabilitado })
}

/// Every top-level menu of `ventana` with its entries, up to the point the
/// scan wraps around to the first one. Submenus are listed but not entered.
pub(crate) fn listar(ventana: Hwnd) -> Result<Value> {
    let (menus, fin) = leer_barra(ventana)?;
    let listado: Vec<Value> = menus
        .iter()
        .enumerate()
        .map(|(i, items)| {
            json!({
                "menu": i,
                "sistema": es_menu_de_sistema(items),
                "items": items.iter().enumerate().map(|(p, it)| item_json(p, it)).collect::<Vec<_>>(),
            })
        })
        .collect();
    let (completo, error) = match fin {
        Fin::Sistema => (true, None),
        Fin::Fallo(motivo) => (false, Some(motivo)),
        Fin::Limite => (false, Some(format!("más de {MAX_MENUS} menús"))),
    };
    Ok(json!({
        "ventana": format!("0x{ventana:X}"),
        "menus": listado,
        "completo": completo,
        "error": error,
    }))
}
