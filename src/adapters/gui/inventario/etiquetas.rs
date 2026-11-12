//! The label of a control and where it was read. Order of trust: the control's
//! own window text, then the name UI Automation gives it, then the text of a
//! window beside it, then the text UI Automation draws beside it.

use super::dominio::Arbol;
use super::tipos::{FuenteEtiqueta, Instantanea, NodoUia, NodoWin32, Rol};
use crate::adapters::gui::ffi::Hwnd;
use crate::adapters::gui::ocr::dominio::{asociar_etiqueta, normalizar_etiqueta, LineaOcr};
use std::collections::HashMap;

/// A control to label, with what UI Automation says of the same window.
pub(super) struct Candidato<'a> {
    pub(super) nodo: &'a NodoWin32,
    pub(super) rol: Rol,
    pub(super) uia: Option<&'a NodoUia>,
}

/// The texts that can name a field by being next to it.
pub(super) struct Rotulos {
    /// Static texts of each page (`None` for the form outside any page).
    por_pagina: HashMap<Option<Hwnd>, Vec<LineaOcr>>,
    /// Text elements of the UI Automation tree.
    uia: Vec<LineaOcr>,
}

fn linea(texto: &str, rect: crate::adapters::gui::ffi::Rect) -> LineaOcr {
    LineaOcr {
        texto: texto.to_string(),
        rect,
        palabras: Vec::new(),
    }
}

impl Rotulos {
    pub(super) fn de(instantanea: &Instantanea, arbol: &Arbol<'_>) -> Self {
        let mut por_pagina: HashMap<Option<Hwnd>, Vec<LineaOcr>> = HashMap::new();
        let textos = instantanea
            .nodos
            .iter()
            .filter(|n| Rol::de_clase(&n.clase) == Rol::Texto && !n.texto.trim().is_empty());
        for n in textos {
            let pagina = arbol.pagina_de(n).map(|p| p.hwnd);
            por_pagina
                .entry(pagina)
                .or_default()
                .push(linea(&n.texto, n.rect));
        }
        let uia = instantanea
            .uia
            .iter()
            .filter(|u| u.rol == "text" && !u.nombre.trim().is_empty())
            .filter_map(|u| Some(linea(&u.nombre, u.rect?)))
            .collect();
        Self { por_pagina, uia }
    }

    fn de_la_pagina(&self, pagina: Option<Hwnd>) -> &[LineaOcr] {
        self.por_pagina.get(&pagina).map_or(&[], Vec::as_slice)
    }
}

/// The name UI Automation gives the control, unless it is the control's own
/// text: a field's value is not its label.
fn nombre_uia(c: &Candidato<'_>) -> Option<String> {
    let nombre = c.uia?.nombre.trim();
    (!nombre.is_empty() && nombre != c.nodo.texto.trim()).then(|| normalizar_etiqueta(nombre))
}

/// The label of `c` and where it came from. `pagina` is the page that holds it.
pub(super) fn etiquetar(
    c: &Candidato<'_>,
    pagina: Option<Hwnd>,
    rotulos: &Rotulos,
) -> (Option<String>, Option<FuenteEtiqueta>) {
    if c.rol.rotula_con_su_texto() && !c.nodo.texto.trim().is_empty() {
        let propio = normalizar_etiqueta(&c.nodo.texto);
        return (Some(propio), Some(FuenteEtiqueta::TextoDeVentana));
    }
    if !c.rol.espera_rotulo_vecino() {
        return (None, None);
    }
    if let Some(nombre) = nombre_uia(c) {
        return (Some(nombre), Some(FuenteEtiqueta::NombreUia));
    }
    let vecino = asociar_etiqueta(&c.nodo.rect, rotulos.de_la_pagina(pagina)).etiqueta;
    if let Some(texto) = vecino {
        return (Some(texto), Some(FuenteEtiqueta::TextoDeVentana));
    }
    // UI Automation sees only what is on screen: a hidden control has no text beside it there.
    if !c.nodo.visible {
        return (None, None);
    }
    match asociar_etiqueta(&c.nodo.rect, &rotulos.uia).etiqueta {
        Some(texto) => (Some(texto), Some(FuenteEtiqueta::NombreUia)),
        None => (None, None),
    }
}
