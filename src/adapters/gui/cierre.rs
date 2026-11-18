//! What a close leaves behind. Closing an editor can make its parent selector
//! reappear as a modal, so the answer is built only after re-reading the forms.
//! A close can also stop at «¿Quiere cancelar las modificaciones?»: [`descarte`]
//! decides whether that exact question is on screen, and which button answers it.

use super::acciones::{clic_por_mensajes, CLASE_MENSAJE};
use super::conduccion::{esperar_cierre, formularios};
use super::ffi::{self, Hwnd};
use super::idioma_gui;
use super::modal;
use super::ocr::captura_ocr::lineas_vivas;
use super::ocr::ubicacion::plegar;
use super::proceso::pids_pvsyst;
use super::rotulos::dialogos::{DESCARTAR_CAMBIOS, SI};
use super::rotulos::{IdiomaGui, Rotulo};
use super::ventanas::{class_of, hijo_visible_con_texto, pid_of, text_of, visible};
use crate::error::{Error, Result};
use crate::idioma::{Idioma, Texto};
use serde_json::{json, Value};
use std::time::Duration;

/// Time PVsyst takes to show the parent form again after an editor closes.
pub(crate) const ESPERA_REAPARICION: Duration = Duration::from_millis(400);
/// Longest wait for a form to disappear after asking it to close.
const ESPERA_CIERRE: Duration = Duration::from_secs(2);
/// The home screen is always behind: it is not a form the close reopened.
const INICIO: &str = "THomeForm";

/// Note of a `descartar_cambios` that could not tell the question apart.
const SIN_IDIOMA: Texto = Texto {
    es: "«descartar_cambios» no respondió: la GUI de PVsyst está en un idioma cuyos rótulos este servidor no conoce (es, en, fr, de), así que no distingue esta pregunta de otra. Respóndela con gui_actuar accion:clic_texto",
    en: "'descartar_cambios' answered nothing: the PVsyst GUI is in a language whose captions this server does not know (es, en, fr, de), so it cannot tell this question from another. Answer it with gui_actuar accion:clic_texto",
};
const SIN_LECTURA: Texto = Texto {
    es: "«descartar_cambios» no respondió: no se pudo leer el texto del diálogo (OCR), así que no se sabe si es la pregunta de descarte. Léelo con gui_observar que:formulario y responde con gui_actuar accion:click",
    en: "'descartar_cambios' answered nothing: the dialog's text could not be read (OCR), so whether it is the discard question is unknown. Read it with gui_observar que:formulario and answer with gui_actuar accion:click",
};

/// What a dialog that intercepted a close is, and whether it was answered.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Descarte {
    /// The dialog is the discard question: press this button to answer it yes.
    Descartar { boton: Rotulo<'static> },
    /// A different dialog: it is left for the caller, as it always was.
    OtroDialogo,
    /// The wording of the question is not known in the GUI's language, so the
    /// dialog cannot be told from another one: nothing is answered.
    IdiomaDesconocido,
    /// The dialog's text could not be read: nothing is answered either.
    SinLectura,
}

/// Whether `texto`, read from a dialog of class `clase`, is the question a close
/// asks before dropping the window's changes, and the button that answers it.
///
/// The whole sentence has to match: the 3D editor asks «¿Quiere mantener sus
/// modificaciones en el objeto?», whose «Sí» adds the object instead of
/// discarding it. Only the Spanish wording was measured on 8.1.8; the other
/// three are inferred and can only fail closed, since an unmeasured wording
/// matches nothing. A language no wording is known in answers nothing.
pub(crate) fn descarte(clase: &str, texto: &str, idioma: Option<IdiomaGui>) -> Descarte {
    if clase != CLASE_MENSAJE {
        return Descarte::OtroDialogo;
    }
    let Some(idioma) = idioma else {
        return Descarte::IdiomaDesconocido;
    };
    let pregunta = plegar(DESCARTAR_CAMBIOS.en_idioma(idioma));
    if plegar(texto).contains(pregunta.as_str()) {
        Descarte::Descartar { boton: SI }
    } else {
        Descarte::OtroDialogo
    }
}

/// A visible top-level window of the process, as seen before and after a close.
pub(crate) struct VentanaViva {
    h: Hwnd,
    clase: String,
    titulo: String,
}

/// The visible forms of `pid` right now.
pub(crate) fn instantanea(pid: u32) -> Vec<VentanaViva> {
    let mut vivas: Vec<VentanaViva> = formularios(pid)
        .into_iter()
        .map(|v| VentanaViva {
            h: v,
            clase: class_of(v),
            titulo: text_of(v),
        })
        .collect();
    // A modal embedded as a child window is not in the top-level list, and it is
    // exactly the form that can block the next call. It can belong to another
    // process of the same executable, so every PVsyst process is searched.
    let mut pids = pids_pvsyst();
    if !pids.contains(&pid) {
        pids.push(pid);
    }
    if let Some(h) = modal::modal_de(&pids) {
        if !vivas.iter().any(|v| v.h == h) {
            vivas.push(VentanaViva {
                h,
                clase: class_of(h),
                titulo: text_of(h),
            });
        }
    }
    vivas
}

/// Forms the close made visible again: a bare `closed:true` would hide them.
pub(crate) fn reabiertas(
    antes: &[VentanaViva],
    despues: &[VentanaViva],
    cerrada: Hwnd,
) -> Vec<Value> {
    despues
        .iter()
        .filter(|v| v.h != cerrada && v.clase != INICIO && !antes.iter().any(|a| a.h == v.h))
        .map(|v| {
            json!({
                "hwnd": format!("0x{:X}", v.h),
                "class": v.clase,
                "title": v.titulo,
            })
        })
        .collect()
}

/// Names the reopened form and the `cerrar` that closes it, so the caller does
/// not have to discover the class with another observation.
pub(crate) fn aviso(reabiertas: &[Value]) -> Option<String> {
    let primera = reabiertas.first()?;
    let clase = primera["class"].as_str().unwrap_or_default();
    let hwnd = primera["hwnd"].as_str().unwrap_or_default();
    let resto = reabiertas.len().saturating_sub(1);
    let extra = if resto == 0 {
        String::new()
    } else {
        format!(" (and {resto} more)")
    };
    Some(format!(
        "Closing reopened {clase}{extra}: close it too with gui_actuar accion:cerrar ventana:{hwnd}"
    ))
}

/// A close that left the window open (it asked for a confirmation, or a modal
/// intercepted it) is a failure the caller must see. With `descartar_cambios`
/// the one question that only asks to discard is answered first, and no other.
pub(super) fn exigir_cerrada(h: Hwnd, idioma: Idioma, descartar_cambios: bool) -> Result<()> {
    if esperar_cierre(h, ESPERA_CIERRE) {
        return Ok(());
    }
    let decision = descartar_cambios.then(|| responder_descarte(h)).flatten();
    if matches!(decision, Some(Descarte::Descartar { .. })) && esperar_cierre(h, ESPERA_CIERRE) {
        return Ok(());
    }
    Err(Error::Gui(cierre_fallido(h, idioma, decision)))
}

/// The TMessageForm PVsyst put over `h`, if the close left one.
fn mensaje_de(h: Hwnd) -> Option<Hwnd> {
    modal::modal_de(&[pid_of(h)]).filter(|m| class_of(*m) == CLASE_MENSAJE)
}

/// Answers the question that blocked a close when it is exactly the discard
/// question. The decision comes back so the error can explain a dialog it could
/// not read or tell apart.
fn responder_descarte(h: Hwnd) -> Option<Descarte> {
    let pregunta = mensaje_de(h)?;
    let Some(texto) = texto_del_dialogo(pregunta) else {
        return Some(Descarte::SinLectura);
    };
    let decision = descarte(&class_of(pregunta), &texto, idioma_gui::en_ejecucion());
    if let Descarte::Descartar { boton } = decision {
        pulsar(pregunta, boton);
    }
    Some(decision)
}

/// Text of the dialog as a person reads it: its caption and the lines OCR draws
/// on it. A TMessageForm shows its question in a `TLabel`, which is not a window,
/// so those pixels are the only place the sentence is.
fn texto_del_dialogo(pregunta: Hwnd) -> Option<String> {
    let lineas = match lineas_vivas(pregunta) {
        Ok(lineas) => lineas,
        Err(error) => {
            tracing::warn!(%error, "no se pudo leer el texto del diálogo de cierre");
            return None;
        }
    };
    let mut texto = text_of(pregunta);
    for linea in lineas {
        texto.push(' ');
        texto.push_str(&linea.texto);
    }
    Some(texto)
}

/// Presses the answer on `pregunta`. A button that is not there, or a click
/// Windows refused, leaves the dialog open and the close failing as before.
fn pulsar(pregunta: Hwnd, boton: Rotulo<'static>) {
    let Some(control) = hijo_visible_con_texto(pregunta, boton) else {
        return;
    };
    if let Err(error) = clic_por_mensajes(control) {
        tracing::warn!(%error, "no se pudo pulsar el botón que descarta los cambios");
    }
}

/// Why the window is still open: the dialog that intercepted the close, with its
/// buttons, plus the note of a `descartar_cambios` that answered nothing.
fn cierre_fallido(h: Hwnd, idioma: Idioma, decision: Option<Descarte>) -> String {
    let pregunta = mensaje_de(h);
    let mut texto = sigue_abierta(&class_of(h), pregunta.map(|m| (m, botones_de(m))), idioma);
    if let Some(nota) = decision.and_then(|d| nota_de(d, idioma)) {
        texto.push(' ');
        texto.push_str(&nota);
    }
    texto
}

/// The note of a `descartar_cambios` that did not answer, and why it did not.
fn nota_de(decision: Descarte, idioma: Idioma) -> Option<String> {
    let texto = match decision {
        Descarte::IdiomaDesconocido => SIN_IDIOMA,
        Descarte::SinLectura => SIN_LECTURA,
        Descarte::Descartar { .. } | Descarte::OtroDialogo => return None,
    };
    Some(texto.en_idioma(idioma).to_string())
}

/// Visible buttons of `h` as `(hwnd, caption)`.
fn botones_de(h: Hwnd) -> Vec<(Hwnd, String)> {
    ffi::enum_children(h)
        .into_iter()
        .filter(|c| visible(*c))
        .map(|c| (c, class_of(c), text_of(c)))
        .filter(|(_, clase, _)| clase.contains("Btn") || clase.contains("Button"))
        .map(|(c, _, texto)| (c, texto))
        .collect()
}

/// Why a close failed: it opened a confirmation (named, with the buttons that
/// answer it) or it is unknown (GUI-14).
fn sigue_abierta(
    clase: &str,
    confirmacion: Option<(Hwnd, Vec<(Hwnd, String)>)>,
    idioma: Idioma,
) -> String {
    let Some((mensaje, botones)) = confirmacion else {
        let abierta = Texto {
            es: "{clase} sigue abierta tras pedirle que se cierre: puede haber abierto una confirmación (gui_observar que:ventanas)",
            en: "{clase} is still open after asking it to close: it may have opened a confirmation (gui_observar que:ventanas)",
        };
        return abierta.texto_con(idioma, &[("clase", clase)]);
    };
    let botones: Vec<String> = botones
        .iter()
        .map(|(b, texto)| format!("'{texto}' (0x{b:X})"))
        .collect();
    let pide_confirmacion = Texto {
        es: "{clase} sigue abierta: pidió confirmación en {mensaje}. Respóndela con gui_actuar accion:click ventana:<hwnd> sobre uno de sus botones: {botones}",
        en: "{clase} is still open: it asked for a confirmation in {mensaje}. Answer it with gui_actuar accion:click ventana:<hwnd> on one of its buttons: {botones}",
    };
    pide_confirmacion.texto_con(
        idioma,
        &[
            ("clase", clase),
            ("mensaje", &format!("{CLASE_MENSAJE} 0x{mensaje:X}")),
            ("botones", &botones.join(", ")),
        ],
    )
}

#[cfg(test)]
#[path = "../../../tests/unit/adapters/gui/cierre.rs"]
mod tests;
