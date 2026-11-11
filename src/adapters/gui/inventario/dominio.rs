//! Pure domain of the GUI inventory: which tab page holds each control and what
//! label it carries. It lists, it does not classify: an unknown class is still
//! a control, with the role `other`.
//!
//! The texts are the ones the controls report when they are read; no caption is
//! written here.

use super::tipos::{
    Ancla, Control, FuenteEtiqueta, Instantanea, Inventario, NodoWin32, Pestana, Rol, CLASE_PAGINA,
};
use crate::adapters::gui::ffi::Hwnd;
use crate::adapters::gui::ocr::dominio::{asociar_etiqueta, normalizar_etiqueta, LineaOcr};
use crate::adapters::gui::posicion::posicion_de_diseno;
use std::collections::HashMap;

fn hex(h: Hwnd) -> String {
    format!("0x{h:X}")
}

/// Nodes by handle: how a control finds its parents.
struct Arbol<'a> {
    por_hwnd: HashMap<Hwnd, &'a NodoWin32>,
}

impl<'a> Arbol<'a> {
    fn de(nodos: &'a [NodoWin32]) -> Self {
        Self {
            por_hwnd: nodos.iter().map(|n| (n.hwnd, n)).collect(),
        }
    }

    /// The nearest page of a page control that holds `nodo`. A chain longer
    /// than the nodes themselves is a cycle and ends the search.
    fn pagina_de(&self, nodo: &NodoWin32) -> Option<&'a NodoWin32> {
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

    /// Static texts of a page (or of the form outside any page), as lines to
    /// pair with the fields beside them.
    fn rotulos_de(&self, arbol: &Arbol<'_>, pagina: Option<Hwnd>) -> Vec<LineaOcr> {
        self.nodos
            .iter()
            .filter(|n| Rol::de_clase(&n.clase) == Rol::Texto && !n.texto.trim().is_empty())
            .filter(|n| arbol.pagina_de(n).map(|p| p.hwnd) == pagina)
            .map(|n| LineaOcr {
                texto: n.texto.clone(),
                rect: n.rect,
                palabras: Vec::new(),
            })
            .collect()
    }
}

/// The label of `nodo` and where it came from.
fn etiquetar(
    nodo: &NodoWin32,
    rol: Rol,
    vecinos: &[LineaOcr],
) -> (Option<String>, Option<FuenteEtiqueta>) {
    if rol.rotula_con_su_texto() && !nodo.texto.trim().is_empty() {
        return (
            Some(normalizar_etiqueta(&nodo.texto)),
            Some(FuenteEtiqueta::TextoDeVentana),
        );
    }
    if !rol.espera_rotulo_vecino() {
        return (None, None);
    }
    match asociar_etiqueta(&nodo.rect, vecinos).etiqueta {
        Some(e) => (Some(e), Some(FuenteEtiqueta::TextoDeVentana)),
        None => (None, None),
    }
}

fn fila(nodo: &NodoWin32, instantanea: &Instantanea, contexto: &ContextoFila<'_>) -> Control {
    let rol = Rol::de_clase(&nodo.clase);
    let pagina = contexto.arbol.pagina_de(nodo);
    let rotulos = instantanea.rotulos_de(&contexto.arbol, pagina.map(|p| p.hwnd));
    let (etiqueta, etiqueta_fuente) = etiquetar(nodo, rol, &rotulos);
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
        marcado: nodo.marcado,
        campo: rol.es_rellenable().then(|| Ancla {
            clase: nodo.clase.clone(),
            x,
            y,
        }),
    }
}

struct ContextoFila<'a> {
    arbol: Arbol<'a>,
    indices: HashMap<Hwnd, usize>,
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

/// Every window of the form as a row, and the tab pages that hold them. O(n log n).
pub(crate) fn inventariar(instantanea: &Instantanea) -> Inventario {
    let contexto = ContextoFila {
        arbol: Arbol::de(&instantanea.nodos),
        indices: indices_por_clase(&instantanea.nodos),
    };
    let controles: Vec<Control> = instantanea
        .nodos
        .iter()
        .map(|n| fila(n, instantanea, &contexto))
        .collect();
    let pestanas = pestanas(instantanea, &contexto.arbol);
    Inventario {
        controles,
        pestanas,
    }
}

#[cfg(test)]
#[path = "../../../../tests/unit/adapters/gui/inventario/dominio.rs"]
mod tests;
