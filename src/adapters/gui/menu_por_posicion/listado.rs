//! `gui_observar que:menus`: every menu of a window's bar with its entries.

use super::{leer_barra, Fin, MAX_MENUS};
use crate::adapters::gui::ffi::{Hwnd, ItemMenu, TipoItem};
use crate::adapters::gui::ventanas::class_of;
use crate::error::Result;
use crate::idioma::{Idioma, Texto};
use serde_json::{json, Value};

fn item_json(posicion: usize, item: &ItemMenu) -> Value {
    let (tipo, id) = match item.tipo {
        TipoItem::Comando(id) => ("comando", Some(id)),
        TipoItem::Separador => ("separador", None),
        TipoItem::Submenu => ("submenu", None),
    };
    json!({ "posicion": posicion, "tipo": tipo, "id": id, "deshabilitado": item.deshabilitado })
}

/// `item_json` plus the catalogued function behind the entry, when there is one:
/// PVsyst's menus are owner-drawn and carry no text, so the catalogue names them (GUI-12).
fn item_con_funcion(clase: &str, (menu, posicion): (usize, usize), item: &ItemMenu) -> Value {
    let mut json = item_json(posicion, item);
    let id = match item.tipo {
        TipoItem::Comando(id) => Some(id),
        TipoItem::Separador | TipoItem::Submenu => None,
    };
    let conocida = crate::adapters::gui::catalogo::funcion_de_menu(clase, (menu, posicion), id);
    if let (Some((clave, etiqueta)), Some(mapa)) = (conocida, json.as_object_mut()) {
        mapa.insert("funcion".into(), Value::from(clave));
        mapa.insert("etiqueta".into(), Value::from(etiqueta));
    }
    json
}

/// Every top-level menu of `ventana` with its entries, up to the point the
/// scan wraps around to the first one. Submenus are listed but not entered.
/// The window's system menu is not listed: `menu` does not accept it (GUI-11).
pub(crate) fn listar(ventana: Hwnd, idioma: Idioma) -> Result<Value> {
    let (mut menus, fin) = leer_barra(ventana)?;
    if matches!(fin, Fin::Sistema) {
        menus.pop();
    }
    let clase = class_of(ventana);
    let listado: Vec<Value> = menus
        .iter()
        .enumerate()
        .map(|(i, items)| {
            let items: Vec<Value> = items
                .iter()
                .enumerate()
                .map(|(p, it)| item_con_funcion(&clase, (i, p), it))
                .collect();
            json!({ "menu": i, "items": items })
        })
        .collect();
    let (completo, error) = match fin {
        Fin::Sistema => (true, None),
        Fin::Fallo(motivo) => (false, Some(motivo)),
        Fin::Limite => {
            let demasiados = Texto {
                es: "más de {max} menús",
                en: "more than {max} menus",
            };
            (
                false,
                Some(demasiados.texto_con(idioma, &[("max", &MAX_MENUS.to_string())])),
            )
        }
    };
    Ok(json!({
        "ventana": format!("0x{ventana:X}"),
        "menus": listado,
        "completo": completo,
        "error": error,
    }))
}
