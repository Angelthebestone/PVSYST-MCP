//! What the catalogue says about each PVsyst function: how to open it, what it
//! does to the user's data and which window it is expected to show.

use crate::args::opt_u32;
use crate::error::{Error, Result};
use serde_json::{json, Value};

/// Window class of the main form; its menu bar and buttons are the entry points.
pub(crate) const VENTANA_INICIO: &str = "THomeForm";

/// How a catalogued function is reached once its host window is open.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Acceso {
    /// Menu item identified by its command id. PVsyst draws its menus itself
    /// (owner-drawn); the main form's ids are stable between runs.
    Menu { id: u16 },
    /// Menu item named by its place in the menu bar (menu, item). The command
    /// ids of a secondary form's menu change with the instance, so its id is
    /// read from the open popup at run time (`gui_observar que:menus`).
    MenuPosicion { menu: u8, item: u8 },
    /// Button of the host form, identified by its caption in the Spanish interface.
    Boton { titulo: &'static str },
}

/// What invoking the function can do beyond opening a window.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Efecto {
    /// Opens a window or dialog; nothing is written until the user confirms there.
    Abre,
    /// Starts an import, export, simulation or list reset: it changes data or
    /// consumes a trial run once confirmed.
    Modifica,
    /// Closes PVsyst.
    Cierra,
}

impl Efecto {
    pub(crate) fn nombre(self) -> &'static str {
        match self {
            Self::Abre => "abre",
            Self::Modifica => "modifica",
            Self::Cierra => "cierra",
        }
    }
}

/// One PVsyst function reachable from the GUI.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct Funcion {
    /// Stable dotted key, e.g. `configuraciones.preferencias`.
    pub(crate) clave: &'static str,
    /// Spanish description of the menu item or button.
    pub(crate) etiqueta: &'static str,
    /// Class of the window that holds the entry point.
    pub(crate) ventana: &'static str,
    pub(crate) acceso: Acceso,
    /// Class of the window it opens, when it was observed; `None` when not yet.
    pub(crate) abre: Option<&'static str>,
    /// Key of the function that opens the host window, when that window is not
    /// the main form (a tool button needs its tools window open first).
    pub(crate) requiere: Option<&'static str>,
    /// Tab page of the host form that holds the button (from 0), when it is
    /// not on the first one.
    pub(crate) pagina: Option<u8>,
    /// Title of the host window, when several windows of its class exist (the
    /// component choosers share `TChoixObjetPVForm`): the host is the visible
    /// window of that class whose title is exactly this.
    pub(crate) titulo_anfitrion: Option<&'static str>,
    /// What could not be checked or why the entry is incomplete, e.g. a flow
    /// that needs a licence PVsyst's trial mode lacks.
    pub(crate) nota: Option<&'static str>,
    pub(crate) efecto: Efecto,
}

impl Funcion {
    /// Entry on the main form; [`Self::en`] moves it to another window.
    const fn nueva(clave: &'static str, etiqueta: &'static str, acceso: Acceso) -> Self {
        Self {
            clave,
            etiqueta,
            ventana: VENTANA_INICIO,
            acceso,
            abre: None,
            requiere: None,
            pagina: None,
            titulo_anfitrion: None,
            nota: None,
            efecto: Efecto::Abre,
        }
    }
    /// Item of the main form's menu bar, by command id.
    pub(crate) const fn menu(clave: &'static str, etiqueta: &'static str, id: u16) -> Self {
        Self::nueva(clave, etiqueta, Acceso::Menu { id })
    }

    /// Item `item` of menu `menu` (both from 0, separators counted) of the main
    /// form's menu bar; move it to another window with [`Self::en`].
    pub(crate) const fn posicion(
        clave: &'static str,
        etiqueta: &'static str,
        (menu, item): (u8, u8),
    ) -> Self {
        Self::nueva(clave, etiqueta, Acceso::MenuPosicion { menu, item })
    }

    /// Button of `ventana` whose caption is `titulo`.
    pub(crate) const fn boton(
        clave: &'static str,
        ventana: &'static str,
        titulo: &'static str,
    ) -> Self {
        Self::nueva(clave, titulo, Acceso::Boton { titulo }).en(ventana)
    }
    /// Records the class of the window the function was seen to open.
    pub(crate) const fn abre(mut self, clase: &'static str) -> Self {
        self.abre = Some(clase);
        self
    }

    /// Records the function that must be run first to open the host window.
    pub(crate) const fn requiere(mut self, clave: &'static str) -> Self {
        self.requiere = Some(clave);
        self
    }

    /// Records the tab page of the host form that holds the entry point.
    pub(crate) const fn en_pagina(mut self, pagina: u8) -> Self {
        self.pagina = Some(pagina);
        self
    }

    /// Records the exact title of the host window (see `titulo_anfitrion`).
    pub(crate) const fn en_ventana_titulada(mut self, titulo: &'static str) -> Self {
        self.titulo_anfitrion = Some(titulo);
        self
    }

    /// Records what could not be checked about the entry.
    pub(crate) const fn nota(mut self, nota: &'static str) -> Self {
        self.nota = Some(nota);
        self
    }

    /// Moves the entry point to another window (the project window has its own
    /// menu bar).
    pub(crate) const fn en(mut self, ventana: &'static str) -> Self {
        self.ventana = ventana;
        self
    }

    /// Overrides the default [`Efecto::Abre`].
    pub(crate) const fn con_efecto(mut self, efecto: Efecto) -> Self {
        self.efecto = efecto;
        self
    }

    /// JSON view returned by `gui_observar que:funciones`.
    pub(crate) fn a_json(&self) -> Value {
        let (via, destino) = match self.acceso {
            Acceso::Menu { id } => ("menu", json!({ "id": id })),
            Acceso::MenuPosicion { menu, item } => {
                ("menu_posicion", json!({ "menu": menu, "item": item }))
            }
            Acceso::Boton { titulo } => ("boton", json!({ "titulo": titulo })),
        };
        json!({
            "clave": self.clave,
            "etiqueta": self.etiqueta,
            "ventana": self.ventana,
            "via": via,
            "destino": destino,
            "abre": self.abre,
            "requiere": self.requiere,
            "pagina": self.pagina,
            "titulo_anfitrion": self.titulo_anfitrion,
            "nota": self.nota,
            "efecto": self.efecto.nombre(),
        })
    }
}

/// How `gui_actuar accion:menu` names the command to run.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum PedidoMenu {
    Id(u16),
    Posicion {
        menu: usize,
        item: usize,
    },
    /// An entry of the submenu at (`menu`, `item`).
    Submenu {
        menu: usize,
        item: usize,
        sub: usize,
    },
}

/// The command a request for `accion:menu` names: an `id`, or a `menu` and an
/// `item` (positions from 0), plus a `subitem` when `item` is a submenu. Giving
/// both kinds, or half a position, is an error rather than a guess.
pub(crate) fn pedido_de_menu(args: &Value) -> Result<PedidoMenu> {
    let (id, menu, item, sub) = (
        opt_u32(args, "id")?,
        opt_u32(args, "menu")?,
        opt_u32(args, "item")?,
        opt_u32(args, "subitem")?,
    );
    match (id, menu, item, sub) {
        (Some(id), None, None, None) => u16::try_from(id)
            .map(PedidoMenu::Id)
            .map_err(|_| Error::Invalid("'id' va de 0 a 65535".into())),
        (None, Some(menu), Some(item), None) => Ok(PedidoMenu::Posicion {
            menu: menu as usize,
            item: item as usize,
        }),
        (None, Some(menu), Some(item), Some(sub)) => Ok(PedidoMenu::Submenu {
            menu: menu as usize,
            item: item as usize,
            sub: sub as usize,
        }),
        _ => Err(Error::Invalid(
            "accion 'menu' requiere 'id', o la pareja 'menu' e 'item' (posiciones desde 0; 'subitem' si 'item' es un submenú), no ambas".into(),
        )),
    }
}

/// Function with this key, or the keys that start with `clave.` when none
/// matches exactly (so `archivo` lists the whole File menu). O(n).
pub(crate) fn buscar<'a>(
    funciones: impl Iterator<Item = &'a Funcion>,
    clave: &str,
) -> Vec<&'a Funcion> {
    let prefijo = format!("{clave}.");
    let candidatas: Vec<&Funcion> = funciones
        .filter(|f| f.clave == clave || f.clave.starts_with(&prefijo))
        .collect();
    match candidatas.iter().find(|f| f.clave == clave) {
        Some(exacta) => vec![*exacta],
        None => candidatas,
    }
}
