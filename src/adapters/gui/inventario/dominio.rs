//! Pure domain of the GUI inventory: which tab page holds each control, what
//! label it carries and what UI Automation adds. It lists, it does not
//! classify: an unknown class is still a control, with the role `other`.
//!
//! The texts are the ones the controls report when they are read; no caption is
//! written here.

use super::etiquetas::{etiquetar, Candidato, Rotulos};
use super::tipos::{
    Ancla, Control, ControlUia, Fuente, FuenteEtiqueta, Instantanea, Inventario, NodoUia,
    NodoWin32, Pestana, Rol, CLASE_PAGINA,
};
use crate::adapters::gui::ffi::{Hwnd, Rect};
use crate::adapters::gui::ocr::dominio::normalizar_etiqueta;
use crate::adapters::gui::posicion::posicion_de_diseno;
use std::collections::HashMap;

fn hex(h: Hwnd) -> String {
    format!("0x{h:X}")
}

/// Nodes by handle: how a control finds its parents.
pub(super) struct Arbol<'a> {
    pub(super) por_hwnd: HashMap<Hwnd, &'a NodoWin32>,
}

impl<'a> Arbol<'a> {
    fn de(nodos: &'a [NodoWin32]) -> Self {
        Self {
            por_hwnd: nodos.iter().map(|n| (n.hwnd, n)).collect(),
        }
    }

    /// The nearest page of a page control that holds `nodo`. A chain longer
    /// than the nodes themselves is a cycle and ends the search.
    pub(super) fn pagina_de(&self, nodo: &NodoWin32) -> Option<&'a NodoWin32> {
        let sube = |n: &&'a NodoWin32| self.por_hwnd.get(&n.padre).copied();
        std::iter::successors(self.por_hwnd.get(&nodo.padre).copied(), sube)
            .take(self.por_hwnd.len())
            .find(|n| n.clase == CLASE_PAGINA)
    }
}

/// Position of each control among those of its class, top to bottom, left to right.
fn indices_por_clase(nodos: &[NodoWin32]) -> HashMap<Hwnd, usize> {
    let mut por_clase: HashMap<&str, Vec<&NodoWin32>> = HashMap::new();
    for n in nodos {
        por_clase.entry(n.clase.as_str()).or_default().push(n);
    }
    let mut indices = HashMap::new();
    for grupo in por_clase.values_mut() {
        grupo.sort_by_key(|n| (n.rect.top, n.rect.left, n.hwnd));
        indices.extend(grupo.iter().enumerate().map(|(i, n)| (n.hwnd, i)));
    }
    indices
}

/// Caption of a tab page, comparable with the one the tab strip reports.
fn nombre_de_pagina(pagina: &NodoWin32) -> String {
    normalizar_etiqueta(&pagina.texto)
}

impl Instantanea {
    /// Index of the page in the tab strip: by its caption among the tab items,
    /// or the current page when it is the only one shown.
    fn indice_de(&self, pagina: &NodoWin32) -> Option<usize> {
        let nombre = nombre_de_pagina(pagina);
        let por_nombre = self
            .pestanas_uia
            .iter()
            .position(|p| normalizar_etiqueta(p) == nombre);
        por_nombre.or(if pagina.visible {
            self.seleccionada
        } else {
            None
        })
    }

    fn es_seleccionada(&self, pagina: &NodoWin32) -> bool {
        match (self.indice_de(pagina), self.seleccionada) {
            (Some(i), Some(s)) => i == s,
            _ => pagina.visible,
        }
    }
}

/// What the rows are built against.
struct ContextoFila<'a> {
    arbol: Arbol<'a>,
    indices: HashMap<Hwnd, usize>,
    uia_por_hwnd: HashMap<Hwnd, &'a NodoUia>,
    rotulos: Rotulos,
}

fn fila(nodo: &NodoWin32, instantanea: &Instantanea, contexto: &ContextoFila<'_>) -> Control {
    let rol = Rol::de_clase(&nodo.clase);
    let pagina = contexto.arbol.pagina_de(nodo);
    let uia = contexto.uia_por_hwnd.get(&nodo.hwnd).copied();
    let candidato = Candidato { nodo, rol, uia };
    let (etiqueta, etiqueta_fuente) =
        etiquetar(&candidato, pagina.map(|p| p.hwnd), &contexto.rotulos);
    let (x, y) = posicion_de_diseno(instantanea.origen, nodo.rect, instantanea.escala);
    Control {
        clase: nodo.clase.clone(),
        hwnd: hex(nodo.hwnd),
        rect: nodo.rect,
        texto: nodo.texto.clone(),
        visible: nodo.visible,
        enabled: nodo.habilitado,
        parent: (nodo.padre != 0).then(|| hex(nodo.padre)),
        pestana: pagina.map(nombre_de_pagina),
        indice_pestana: pagina.and_then(|p| instantanea.indice_de(p)),
        rol,
        indice_clase: contexto.indices.get(&nodo.hwnd).copied().unwrap_or(0),
        etiqueta,
        etiqueta_fuente,
        fuente: if uia.is_some() {
            Fuente::Fusionada
        } else {
            Fuente::Win32
        },
        patrones: uia.map(|u| u.patrones.clone()).unwrap_or_default(),
        marcado: nodo.marcado,
        campo: rol.es_rellenable().then(|| Ancla {
            clase: nodo.clase.clone(),
            x,
            y,
        }),
    }
}

fn pestanas(instantanea: &Instantanea, arbol: &Arbol<'_>) -> Vec<Pestana> {
    instantanea
        .nodos
        .iter()
        .filter(|n| n.clase == CLASE_PAGINA)
        .map(|pagina| {
            let contenido: Vec<String> = instantanea
                .nodos
                .iter()
                .filter(|n| arbol.pagina_de(n).map(|p| p.hwnd) == Some(pagina.hwnd))
                .map(|n| hex(n.hwnd))
                .collect();
            let seleccionada = instantanea.es_seleccionada(pagina);
            Pestana {
                indice: instantanea.indice_de(pagina),
                nombre: nombre_de_pagina(pagina),
                hwnd: hex(pagina.hwnd),
                seleccionada,
                creado_al_activar: !seleccionada && contenido.is_empty(),
                controles: contenido,
            }
        })
        .collect()
}

fn contiene(r: &Rect, (x, y): (i32, i32)) -> bool {
    x >= r.left && x <= r.right && y >= r.top && y <= r.bottom
}

/// The page that is shown, when there is exactly one: what UI Automation draws sits on it.
fn pagina_mostrada(nodos: &[NodoWin32]) -> Option<&NodoWin32> {
    let mut mostradas = nodos
        .iter()
        .filter(|n| n.clase == CLASE_PAGINA && n.visible);
    match (mostradas.next(), mostradas.next()) {
        (Some(pagina), None) => Some(pagina),
        _ => None,
    }
}

fn fila_uia(u: &NodoUia, rect: Rect, pagina: Option<&NodoWin32>) -> ControlUia {
    let centro = ((rect.left + rect.right) / 2, rect.centro_y());
    let nombre = u.nombre.trim();
    ControlUia {
        clase: u.clase.clone(),
        hwnd: u.hwnd.map(hex),
        rect,
        texto: u.nombre.clone(),
        visible: None,
        enabled: None,
        parent: None,
        pestana: pagina
            .filter(|p| contiene(&p.rect, centro))
            .map(nombre_de_pagina),
        rol: u.rol.clone(),
        etiqueta: (!nombre.is_empty()).then(|| normalizar_etiqueta(nombre)),
        etiqueta_fuente: (!nombre.is_empty()).then_some(FuenteEtiqueta::NombreUia),
        fuente: Fuente::Uia,
        patrones: u.patrones.clone(),
        solo_vision: u.solo_vision,
    }
}

/// The elements UI Automation sees and Win32 has no window for: the cells,
/// labels and items a control draws itself.
fn solo_uia(instantanea: &Instantanea, arbol: &Arbol<'_>) -> Vec<ControlUia> {
    let pagina = pagina_mostrada(&instantanea.nodos);
    instantanea
        .uia
        .iter()
        .filter(|u| u.hwnd.is_none_or(|h| !arbol.por_hwnd.contains_key(&h)))
        .filter(|u| u.informa())
        .filter_map(|u| Some(fila_uia(u, u.rect?, pagina)))
        .collect()
}

/// Every window of the form as a row, the tab pages that hold them and the
/// elements only UI Automation sees. O(n log n).
pub(crate) fn inventariar(instantanea: &Instantanea) -> Inventario {
    let arbol = Arbol::de(&instantanea.nodos);
    let rotulos = Rotulos::de(instantanea, &arbol);
    let contexto = ContextoFila {
        indices: indices_por_clase(&instantanea.nodos),
        uia_por_hwnd: instantanea
            .uia
            .iter()
            .filter_map(|u| Some((u.hwnd?, u)))
            .collect(),
        rotulos,
        arbol,
    };
    let controles = instantanea
        .nodos
        .iter()
        .map(|n| fila(n, instantanea, &contexto))
        .collect();
    Inventario {
        controles,
        solo_uia: solo_uia(instantanea, &contexto.arbol),
        pestanas: pestanas(instantanea, &contexto.arbol),
    }
}

#[cfg(test)]
#[path = "../../../../tests/unit/adapters/gui/inventario/dominio.rs"]
mod tests;
