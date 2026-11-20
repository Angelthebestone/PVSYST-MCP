//! Catalogue actions against the live PVsyst: the `que:funciones` listing and
//! the `accion:menu` command, with the checks a hand-typed position needs.
//! Opening a catalogued function lives in [`super::abrir`].

use super::dominio::{buscar, pedido_de_menu, Funcion, PedidoMenu};
use super::entradas::funciones;
use super::espera::{esperar_nuevas, SolicitudEspera};
use crate::adapters::gui::ffi::{self, Hwnd, WM_COMMAND};
use crate::adapters::gui::menu_por_posicion::{
    contar_menus, id_en_posicion, id_en_submenu, Posicion,
};
use crate::adapters::gui::modal;
use crate::adapters::gui::ventanas::{class_of, hwnd_de, ventanas_de, visible};
use crate::error::{Error, Result};
use crate::idioma::{Idioma, Texto};
use serde_json::{json, Value};
use std::time::Duration;

const ESPERA_POR_DEFECTO_MS: u64 = 5_000;
/// Longest wait a caller may ask for with `timeout_ms`.
const ESPERA_MAXIMA_MS: u64 = 60_000;

/// `gui_observar que:funciones`: the catalogue, or the keys under `funcion`.
pub(crate) fn listar(args: &Value) -> Value {
    let base: Vec<&Funcion> = match args.get("funcion").and_then(Value::as_str) {
        Some(clave) => buscar(funciones(), clave),
        None => funciones().collect(),
    };
    let ventana = args.get("ventana").and_then(Value::as_str);
    let lista: Vec<Value> = base
        .into_iter()
        .filter(|f| ventana.is_none_or(|v| f.ventana.eq_ignore_ascii_case(v)))
        .map(Funcion::a_json)
        .collect();
    json!({ "count": lista.len(), "funciones": lista })
}

/// `gui_actuar accion:menu`: posts a menu command to `ventana`, named by its
/// command `id` or by its place in the menu bar (`menu` and `item`).
pub(crate) fn menu(args: &Value) -> Result<Value> {
    // The scene importer calls it without a `Paths`: the environment is the
    // same source `Paths` reads.
    let idioma = Idioma::de_entorno();
    let pedido = pedido_de_menu(args, idioma)?;
    let anfitrion = hwnd_de(args, "ventana")?;
    exigir_menu_en_la_barra(anfitrion, pedido, idioma)?;
    let id = id_de_pedido(anfitrion, pedido)?;
    let antes = visibles(args)?;
    enviar_comando(anfitrion, id, idioma)?;
    let visto = esperar_nuevas(
        SolicitudEspera {
            antes: &antes,
            esperada: None,
            limite: espera_pedida(args),
        },
        || visibles(args),
    )?;
    Ok(json!({
        "id": id,
        "ventanas_nuevas": visto.nuevas_json(),
    }))
}

/// A menu number past the end of the bar wraps around to the first menus and
/// would run another command: a request typed by hand is checked against the
/// real bar. Catalogue entries are fixed data, verified live, and skip this.
fn exigir_menu_en_la_barra(anfitrion: Hwnd, pedido: PedidoMenu, idioma: Idioma) -> Result<()> {
    let (PedidoMenu::Posicion { menu, .. } | PedidoMenu::Submenu { menu, .. }) = pedido else {
        return Ok(());
    };
    exigir_anfitrion_habilitado(anfitrion)?;
    let menus = contar_menus(anfitrion)?;
    if menu < menus {
        return Ok(());
    }
    let fuera_de_rango = Texto {
        es: "La barra de {clase} tiene {menus} menús (0..{ultimo}): 'menu' {menu} daría la vuelta",
        en: "The {clase} bar has {menus} menus (0..{ultimo}): 'menu' {menu} would wrap around",
    };
    Err(Error::Invalid(fuera_de_rango.texto_con(
        idioma,
        &[
            ("clase", &class_of(anfitrion)),
            ("menus", &menus.to_string()),
            ("ultimo", &menus.saturating_sub(1).to_string()),
            ("menu", &menu.to_string()),
        ],
    )))
}

pub(super) fn id_de_pedido(anfitrion: Hwnd, pedido: PedidoMenu) -> Result<u16> {
    match pedido {
        PedidoMenu::Id(id) => Ok(id),
        PedidoMenu::Posicion { menu, item } => {
            exigir_anfitrion_habilitado(anfitrion)?;
            id_en_posicion(anfitrion, Posicion { menu, item })
        }
        PedidoMenu::Submenu { menu, item, sub } => {
            exigir_anfitrion_habilitado(anfitrion)?;
            id_en_submenu(anfitrion, Posicion { menu, item }, sub)
        }
    }
}

pub(super) fn visibles(args: &Value) -> Result<Vec<Hwnd>> {
    Ok(ventanas_de(args)?
        .into_iter()
        .filter(|h| visible(*h))
        .collect())
}

/// A VCL modal dialog disables the form it was opened from. A menu command
/// posted to that form still runs, and PVsyst raises an internal exception
/// that forces a restart, so commands are refused while the host is disabled.
pub(super) fn exigir_anfitrion_habilitado(anfitrion: Hwnd) -> Result<()> {
    modal::exigir_habilitada(anfitrion, "sending menu commands")
}

pub(super) fn enviar_comando(anfitrion: Hwnd, id: u16, idioma: Idioma) -> Result<()> {
    exigir_anfitrion_habilitado(anfitrion)?;
    if ffi::post_message(
        anfitrion,
        ffi::Mensaje::nuevo(WM_COMMAND, usize::from(id), 0),
    ) {
        return Ok(());
    }
    let sin_mensaje = Texto {
        es: "PostMessage falló para la ventana 0x{hwnd}",
        en: "PostMessage failed for window 0x{hwnd}",
    };
    Err(Error::Gui(
        sin_mensaje.texto_con(idioma, &[("hwnd", &format!("{anfitrion:X}"))]),
    ))
}

/// The wait the caller asked for, capped.
pub(super) fn espera_pedida(args: &Value) -> Duration {
    let ms = args
        .get("timeout_ms")
        .and_then(Value::as_u64)
        .unwrap_or(ESPERA_POR_DEFECTO_MS);
    Duration::from_millis(ms.min(ESPERA_MAXIMA_MS))
}
