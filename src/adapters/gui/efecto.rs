//! What a click or a key changed. Windows drops input silently and PVsyst
//! answers nothing, so an action that only posts the input would report success
//! whether or not anything happened. The effect is read the way a person would:
//! did a window open or close, did the window repaint.

use super::captura::{rgb_de_control, ImagenRgb};
use super::conduccion::formularios;
use super::ffi::{self, Hwnd};
use super::modal;
use super::ventanas::{class_of, pid_of};
use serde_json::{json, Value};

/// Hint windows come and go with the mouse: they are not what the click did.
const CLASE_AVISO: &str = "THintWindow";

/// The windows of a process and the picture of one of them, at one moment.
pub(crate) struct Instantanea {
    ventanas: Vec<(Hwnd, String)>,
    imagen: Option<ImagenRgb>,
}

impl Instantanea {
    /// The visible forms of the process of `h` (an embedded modal included: it is
    /// not in the top-level list) and a capture of `h`. A capture that fails
    /// (the window closed, the desktop is not interactive) is left out.
    pub(crate) fn de(h: Hwnd) -> Self {
        let pid = pid_of(h);
        let mut ventanas: Vec<(Hwnd, String)> = formularios(pid)
            .into_iter()
            .map(|v| (v, class_of(v)))
            .filter(|(_, clase)| clase != CLASE_AVISO)
            .collect();
        let embebido = modal::modal_de(&[pid]).filter(|m| !ventanas.iter().any(|(v, _)| v == m));
        ventanas.extend(embebido.map(|m| (m, class_of(m))));
        let imagen = ffi::es_ventana(h).then(|| rgb_de_control(h).ok()).flatten();
        Self { ventanas, imagen }
    }
}

/// Classes of the windows of `de` that `contra` does not have.
fn clases_que_faltan(de: &[(Hwnd, String)], contra: &[(Hwnd, String)]) -> Vec<String> {
    de.iter()
        .filter(|(v, _)| !contra.iter().any(|(c, _)| c == v))
        .map(|(_, clase)| clase.clone())
        .collect()
}

/// Whether two captures differ. A missing one cannot be compared: no change is claimed.
fn imagenes_distintas(antes: Option<&ImagenRgb>, despues: Option<&ImagenRgb>) -> bool {
    match (antes, despues) {
        (Some(a), Some(d)) => a.ancho != d.ancho || a.alto != d.alto || a.rgb != d.rgb,
        _ => false,
    }
}

/// What changed between two instants.
// ponytail: whole-window pixel diff; an animation or a blinking caret reads as a
// change. Narrow it to the clicked control if a false `verificado:true` shows up.
#[derive(Debug, Default, PartialEq, Eq)]
pub(crate) struct Efecto {
    pub(crate) abiertas: Vec<String>,
    pub(crate) cerradas: Vec<String>,
    pub(crate) pantalla_cambio: bool,
}

impl Efecto {
    pub(crate) fn entre(antes: &Instantanea, despues: &Instantanea) -> Self {
        Self {
            abiertas: clases_que_faltan(&despues.ventanas, &antes.ventanas),
            cerradas: clases_que_faltan(&antes.ventanas, &despues.ventanas),
            pantalla_cambio: imagenes_distintas(antes.imagen.as_ref(), despues.imagen.as_ref()),
        }
    }

    /// Whether anything at all changed.
    pub(crate) fn hubo(&self) -> bool {
        !self.abiertas.is_empty() || !self.cerradas.is_empty() || self.pantalla_cambio
    }

    pub(crate) fn a_json(&self) -> Value {
        json!({
            "abiertas": self.abiertas,
            "cerradas": self.cerradas,
            "pantalla_cambio": self.pantalla_cambio,
        })
    }
}

#[cfg(test)]
#[path = "../../../tests/unit/adapters/gui/efecto.rs"]
mod tests;
