//! Menu commands addressed by position. PVsyst's menus are owner-drawn, and
//! the command ids of a form's menu depend on the instance (the project
//! window's moved by three between two sessions), so a command is named as
//! (menu, item) and its id is read from the popup while it is open. The form
//! has no `HMENU` of its own (`GetMenu` answers 0), so the popup has to be
//! opened, and that takes the keyboard: VCL ignores posted `WM_KEY*` messages
//! on the private desktop.

use super::escritorio;
use super::ffi::{self, Hwnd, ItemMenu, TipoItem};
use super::ventanas::{class_of, enum_windows_de, pid_of, visible};
use crate::error::{Error, Result};
use serde_json::{json, Value};
use std::time::{Duration, Instant};

const CLASE_POPUP: &str = "#32768";
const VK_ALT: u8 = 0x12;
const VK_IZQUIERDA: u8 = 0x25;
const VK_DERECHA: u8 = 0x27;
const VK_ABAJO: u8 = 0x28;
const VK_ESCAPE: u8 = 0x1B;
/// Escapes that leave the menu bar from a popup (the popup, then the bar).
const NIVELES_DE_MENU: usize = 2;
const PAUSA_TECLA: Duration = Duration::from_millis(120);
const ESPERA_FOCO: Duration = Duration::from_millis(400);
const ESPERA_POPUP: Duration = Duration::from_secs(1);
const SONDEO: Duration = Duration::from_millis(50);
/// Most menus a menu bar is scanned for, and the largest `menu` accepted.
const MAX_MENUS: usize = 16;
/// Command ids from here up belong to the window's system menu.
const PRIMER_ID_DE_SISTEMA: u32 = 0xF000;

/// An item of a menu bar, counted from the left; separators count as items.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct Posicion {
    pub(crate) menu: usize,
    pub(crate) item: usize,
}

/// Why a scan of the menu bar stopped.
enum Fin {
    /// The window's system menu came up: it always follows the last menu of
    /// the bar, so every menu was read.
    Sistema,
    /// A menu failed to open; the listing may be incomplete.
    Fallo(String),
    /// The scan hit [`MAX_MENUS`] without reaching the system menu.
    Limite,
}

/// Keys go to the foreground window: refuse to type unless it belongs to the
/// process of `ventana`. The process, not the window: once the menu bar is
/// active, PVsyst's own popup windows take the foreground.
fn exigir_primer_plano(ventana: Hwnd) -> Result<()> {
    if pid_of(ffi::primer_plano()) == pid_of(ventana) {
        return Ok(());
    }
    Err(Error::Gui(format!(
        "PVsyst perdió el primer plano ({}): no se envían teclas a otra aplicación",
        class_of(ventana)
    )))
}

fn poner_al_frente(ventana: Hwnd) -> Result<()> {
    if escritorio::es_aislado() {
        return Err(Error::Gui(
            "Los menús se abren con el teclado y el escritorio aislado ignora las teclas: usa el botón equivalente (accion:clic_texto) o PVSYST_MCP_ESCRITORIO sin aislar".into(),
        ));
    }
    ffi::traer_al_frente(ventana);
    std::thread::sleep(ESPERA_FOCO);
    exigir_primer_plano(ventana)
}

/// `veces` presses of `vk`, checking before each one that `ventana` still
/// has the focus.
fn pulsar(ventana: Hwnd, vk: u8, veces: usize) -> Result<()> {
    for _ in 0..veces {
        exigir_primer_plano(ventana)?;
        ffi::pulsar_combinacion(vk, &[]);
        std::thread::sleep(PAUSA_TECLA);
    }
    Ok(())
}

fn popup_visible(pid: u32) -> Option<Hwnd> {
    enum_windows_de(pid)
        .into_iter()
        .find(|h| visible(*h) && class_of(*h) == CLASE_POPUP)
}

fn esperar_popup(pid: u32) -> Option<Hwnd> {
    let fin = Instant::now() + ESPERA_POPUP;
    loop {
        if let Some(p) = popup_visible(pid) {
            return Some(p);
        }
        if Instant::now() >= fin {
            return None;
        }
        std::thread::sleep(SONDEO);
    }
}

/// Enters the menu bar and opens top-level menu `menu`. After Alt the first
/// menu is selected but only opens once the selection has moved, so it is
/// reached by stepping off it and back.
fn abrir_menu(ventana: Hwnd, menu: usize) -> Result<()> {
    pulsar(ventana, VK_ALT, 1)?;
    match menu {
        0 => {
            pulsar(ventana, VK_DERECHA, 1)?;
            pulsar(ventana, VK_IZQUIERDA, 1)?;
        }
        _ => pulsar(ventana, VK_DERECHA, menu)?,
    }
    pulsar(ventana, VK_ABAJO, 1)
}

/// Leaves the menu bar. `en_el_popup` says whether a popup was open. VCL's
/// menu bar is its own loop, so Windows cannot say whether it is active:
/// Escape twice closes the popup and then the bar, but sent when the bar is
/// already off it reaches the form, where VCL takes it as Cancel. Without a
/// popup the bar is at most armed by Alt, and Alt again disarms it.
fn salir_de_la_barra(ventana: Hwnd, en_el_popup: bool) {
    let (tecla, veces) = if en_el_popup {
        (VK_ESCAPE, NIVELES_DE_MENU)
    } else {
        (VK_ALT, 1)
    };
    // A lost focus means the keys must not be sent at all.
    let _ = pulsar(ventana, tecla, veces);
}

fn entradas_del_popup(ventana: Hwnd, menu: usize) -> (Result<Vec<ItemMenu>>, bool) {
    if let Err(e) = abrir_menu(ventana, menu) {
        return (Err(e), false);
    }
    let popup = esperar_popup(pid_of(ventana));
    let items = popup.and_then(ffi::items_de_popup).ok_or_else(|| {
        Error::NotFound(format!(
            "El menú {menu} de {} no se abrió",
            class_of(ventana)
        ))
    });
    (items, popup.is_some())
}

/// Opens top-level menu `menu` of `ventana`, reads its entries and always
/// leaves the menu bar, also when something failed half way.
fn leer_menu(ventana: Hwnd, menu: usize) -> Result<Vec<ItemMenu>> {
    poner_al_frente(ventana)?;
    let (items, popup_abierto) = entradas_del_popup(ventana, menu);
    salir_de_la_barra(ventana, popup_abierto);
    items
}

fn exigir_menu_valido(menu: usize) -> Result<()> {
    if menu < MAX_MENUS {
        return Ok(());
    }
    Err(Error::Invalid(format!(
        "'menu' va de 0 a {} (posición en la barra de menús)",
        MAX_MENUS - 1
    )))
}

/// `WM_COMMAND` id of the command at `posicion`, read from the open menu.
pub(crate) fn id_en_posicion(ventana: Hwnd, posicion: Posicion) -> Result<u16> {
    exigir_menu_valido(posicion.menu)?;
    let items = leer_menu(ventana, posicion.menu)?;
    if es_menu_de_sistema(&items) {
        return Err(Error::Invalid(format!(
            "El menú {} es el del sistema de la ventana, no uno de su barra",
            posicion.menu
        )));
    }
    let item = items.get(posicion.item).ok_or_else(|| {
        Error::NotFound(format!(
            "El menú {} solo tiene {} entradas",
            posicion.menu,
            items.len()
        ))
    })?;
    match item.tipo {
        TipoItem::Comando(_) if item.deshabilitado => Err(Error::Gui(format!(
            "La entrada {} del menú {} está deshabilitada",
            posicion.item, posicion.menu
        ))),
        TipoItem::Comando(id) => u16::try_from(id)
            .map_err(|_| Error::Invalid(format!("Id de comando {id} fuera de 16 bits"))),
        _ => Err(Error::Invalid(format!(
            "La entrada {} del menú {} no es un comando (separador o submenú)",
            posicion.item, posicion.menu
        ))),
    }
}

fn item_json(posicion: usize, item: &ItemMenu) -> Value {
    let (tipo, id) = match item.tipo {
        TipoItem::Comando(id) => ("comando", Some(id)),
        TipoItem::Separador => ("separador", None),
        TipoItem::Submenu => ("submenu", None),
    };
    json!({ "posicion": posicion, "tipo": tipo, "id": id, "deshabilitado": item.deshabilitado })
}

fn es_menu_de_sistema(items: &[ItemMenu]) -> bool {
    matches!(items.first(), Some(ItemMenu { tipo: TipoItem::Comando(id), .. }) if *id >= PRIMER_ID_DE_SISTEMA)
}

/// Reads the menus of the bar from the first on, until the window's system
/// menu (always the one after the last) comes up or a menu does not open.
fn leer_barra(ventana: Hwnd) -> Result<(Vec<Vec<ItemMenu>>, Fin)> {
    let mut menus = Vec::new();
    for menu in 0..MAX_MENUS {
        let items = match leer_menu(ventana, menu) {
            Ok(items) => items,
            Err(e) if menu > 0 => return Ok((menus, Fin::Fallo(e.to_string()))),
            Err(e) => return Err(e),
        };
        let es_sistema = es_menu_de_sistema(&items);
        menus.push(items);
        if es_sistema {
            return Ok((menus, Fin::Sistema));
        }
    }
    Ok((menus, Fin::Limite))
}

/// Menus of the bar (without the system menu) of `ventana`.
pub(crate) fn contar_menus(ventana: Hwnd) -> Result<usize> {
    let (menus, fin) = leer_barra(ventana)?;
    match fin {
        Fin::Sistema => Ok(menus.len() - 1),
        Fin::Fallo(motivo) => Err(Error::Gui(format!("No se pudo contar los menús: {motivo}"))),
        Fin::Limite => Err(Error::Gui(format!("Más de {MAX_MENUS} menús"))),
    }
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
