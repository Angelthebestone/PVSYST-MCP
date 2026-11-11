//! Pure domain of the GUI inventory: what a form's controls are, which tab page
//! holds each one and what label it carries. It lists, it does not classify:
//! an unknown class is still a control, with the role `other`.
//!
//! Nothing here reads a caption written by hand: classes, window handles and
//! rectangles are the same in any PVsyst language, and the texts are the ones
//! the controls report when they are read.

use crate::adapters::gui::ffi::{Hwnd, Rect};
use crate::adapters::gui::ocr::dominio::{asociar_etiqueta, normalizar_etiqueta, LineaOcr};
use crate::adapters::gui::posicion::posicion_de_diseno;
use serde::Serialize;
use std::collections::HashMap;

/// VCL class of a page of a page control; its window text is the tab caption.
pub(crate) const CLASE_PAGINA: &str = "TTabSheet";

/// A window of the form as Win32 reports it, hidden ones included.
#[derive(Debug, Clone, PartialEq)]
pub(crate) struct NodoWin32 {
    pub(crate) hwnd: Hwnd,
    pub(crate) padre: Hwnd,
    pub(crate) clase: String,
    pub(crate) texto: String,
    pub(crate) rect: Rect,
    pub(crate) visible: bool,
    pub(crate) habilitado: bool,
    pub(crate) marcado: Option<bool>,
}

/// What a control is for. The names are the ones the UI Automation roles use.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
pub(crate) enum Rol {
    #[serde(rename = "button")]
    Boton,
    #[serde(rename = "checkbox")]
    Casilla,
    #[serde(rename = "radio button")]
    Opcion,
    #[serde(rename = "edit")]
    Campo,
    #[serde(rename = "combo box")]
    Combo,
    #[serde(rename = "list")]
    Lista,
    #[serde(rename = "tree")]
    Arbol,
    #[serde(rename = "tab")]
    Pestanas,
    #[serde(rename = "tab page")]
    Pagina,
    #[serde(rename = "group")]
    Grupo,
    #[serde(rename = "text")]
    Texto,
    #[serde(rename = "pane")]
    Panel,
    #[serde(rename = "other")]
    Otro,
}

const BOTONES: &[&str] = &[
    "TButton",
    "TBitBtn",
    "TPngBitBtn",
    "TSpeedButton",
    "TToolButton",
    "Button",
];
const CAMPOS: &[&str] = &[
    "TEdit",
    "TMemo",
    "TLabeledEdit",
    "TMaskEdit",
    "TSpinEdit",
    "TDateTimePicker",
    "TUpDown",
    "TTrackBar",
];
const LISTAS: &[&str] = &["TListBox", "TCheckListBox", "TListView"];
const ARBOLES: &[&str] = &["TVirtualStringTree", "TTreeView"];
const GRUPOS: &[&str] = &["TGroupBox", "TRadioGroup", "TCheckGroup"];
const PANELES: &[&str] = &[
    "TPanel",
    "TGridPanel",
    "TFlowPanel",
    "TScrollBox",
    "TActionToolBar",
];

impl Rol {
    pub(crate) fn de_clase(clase: &str) -> Self {
        match clase {
            "TCheckBox" => Self::Casilla,
            "TRadioButton" => Self::Opcion,
            "TComboBox" | "TComboBoxEx" => Self::Combo,
            "TPageControl" | "TTabControl" => Self::Pestanas,
            "TStaticText" => Self::Texto,
            CLASE_PAGINA => Self::Pagina,
            c if BOTONES.contains(&c) => Self::Boton,
            c if CAMPOS.contains(&c) => Self::Campo,
            c if LISTAS.contains(&c) => Self::Lista,
            c if ARBOLES.contains(&c) => Self::Arbol,
            c if GRUPOS.contains(&c) => Self::Grupo,
            c if PANELES.contains(&c) => Self::Panel,
            _ => Self::Otro,
        }
    }

    /// Whether the control's own window text is its label.
    fn rotula_con_su_texto(self) -> bool {
        matches!(
            self,
            Self::Boton | Self::Casilla | Self::Opcion | Self::Pagina | Self::Grupo | Self::Texto
        )
    }

    /// Whether a caption drawn next to the control names it.
    fn espera_rotulo_vecino(self) -> bool {
        matches!(self, Self::Campo | Self::Combo | Self::Lista | Self::Arbol)
    }

    /// Whether `gui_actuar` can fill it by its anchor (`campo`).
    fn es_rellenable(self) -> bool {
        matches!(
            self,
            Self::Campo | Self::Combo | Self::Casilla | Self::Opcion | Self::Lista
        )
    }
}

/// Where the label of a control was read.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
pub(crate) enum FuenteEtiqueta {
    /// The control's own window text, or the text of a window next to it.
    #[serde(rename = "window_text")]
    TextoDeVentana,
    /// Text recognised on a capture of the form.
    #[serde(rename = "ocr")]
    Ocr,
}

/// A control located by its class and its position in the form's 96-DPI layout:
/// the way `gui_actuar accion:escribir` and `campo` address a field.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub(crate) struct Ancla {
    pub(crate) clase: String,
    pub(crate) x: i32,
    pub(crate) y: i32,
}

/// One row of the inventory.
#[derive(Clone, Debug, PartialEq, Serialize)]
pub(crate) struct Control {
    pub(crate) clase: String,
    pub(crate) hwnd: String,
    pub(crate) rect: Rect,
    pub(crate) texto: String,
    pub(crate) visible: bool,
    pub(crate) enabled: bool,
    pub(crate) parent: Option<String>,
    pub(crate) pestana: Option<String>,
    pub(crate) indice_pestana: Option<usize>,
    pub(crate) rol: Rol,
    /// Position among the controls of the same class, reading the form top to bottom.
    pub(crate) indice_clase: usize,
    pub(crate) etiqueta: Option<String>,
    pub(crate) etiqueta_fuente: Option<FuenteEtiqueta>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) marcado: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) campo: Option<Ancla>,
}

/// A tab page of the form with the controls it holds.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub(crate) struct Pestana {
    pub(crate) indice: Option<usize>,
    pub(crate) nombre: String,
    pub(crate) hwnd: String,
    pub(crate) seleccionada: bool,
    pub(crate) controles: Vec<String>,
    /// A page that is not shown and holds no window: VCL may build its controls
    /// only when it is first shown, so it can look empty until then.
    pub(crate) creado_al_activar: bool,
}

/// Everything read from a form: its windows and where the form sits.
#[derive(Debug, Clone)]
pub(crate) struct Instantanea {
    pub(crate) nodos: Vec<NodoWin32>,
    /// Window rectangle of the form, the origin of the 96-DPI positions.
    pub(crate) origen: Rect,
    pub(crate) escala: f64,
    /// Tab captions in the order of the tab strip, as UI Automation lists them.
    pub(crate) pestanas_uia: Vec<String>,
    /// Page the page control reports as current.
    pub(crate) seleccionada: Option<usize>,
}

pub(crate) struct Inventario {
    pub(crate) controles: Vec<Control>,
    pub(crate) pestanas: Vec<Pestana>,
}

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
