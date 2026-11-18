//! Modal dialogs. A VCL modal disables every form below it, and Windows drops the
//! input sent to a disabled window without reporting anything, so an action on
//! such a window looks successful and does nothing. Actions refuse those targets.

use super::acciones::CLASE_EXCEPCION;
use super::cierre::ESPERA_CIERRE;
use super::conduccion::{esperar_cierre, formularios, Formulario};
use super::ffi::{self, Hwnd, GA_ROOT};
use super::idioma_gui;
use super::proceso::pids_pvsyst;
use super::rotulos::dialogos::CERRAR;
use super::rotulos::{IdiomaGui, Rotulo};
use super::ventanas::{class_of, enum_windows_de, hijos_visibles, pid_of, text_of, visible};
use crate::error::{Error, Result};
use crate::idioma::{Idioma, Texto};
use serde_json::{json, Value};

/// Class of the standard Windows dialogs (file open/save).
const CLASE_DIALOGO_WINDOWS: &str = "#32770";

/// The DEMO notice PVsyst shows in trial mode («Modo DEMO»): «Compra»,
/// «Activar», «Cerrar», «Ayuda». It only informs, but a VCL modal disables every
/// window below it, so an action that meets it looks refused until someone
/// presses «Cerrar». The other three buy, activate a licence or open the help.
pub(crate) const CLASE_AVISO_DEMO: &str = "TDEMOForm";

/// Hint completing "…before" for actions that click or type into a window.
pub(crate) const PARA_ACTUAR: &str = "acting on it";

/// A VCL modal can be a child window of the form it disables: `EnumWindows`
/// does not list it, so the blocker is looked for among the children first.
fn es_modal_hijo(clase: &str, estado: ffi::EstadoVentana) -> bool {
    estado.visible && estado.habilitada && clase.ends_with("Form")
}

/// Whether a top-level window of this class can be a dialog that blocks a form:
/// a VCL form, the madExcept crash box or a standard Windows dialog. The overlays
/// Windows adds to the process (`UAC_InputIndicatorOverlayWnd`) are enabled and
/// visible but block nothing (GUI-13).
fn puede_bloquear(clase: &str) -> bool {
    clase.ends_with("Form") || clase == CLASE_EXCEPCION || clase == CLASE_DIALOGO_WINDOWS
}

/// The dialog blocking `raiz`: an enabled child form when the modal is embedded,
/// else the enabled top-level window above it.
pub(crate) fn bloqueante_de(raiz: Hwnd) -> Option<Hwnd> {
    if ffi::estado_ventana(raiz).habilitada {
        return None;
    }
    let hijo = ffi::enum_children(raiz)
        .into_iter()
        .find(|c| es_modal_hijo(&class_of(*c), ffi::estado_ventana(*c)));
    hijo.or_else(|| {
        formularios(pid_of(raiz)).into_iter().find(|h| {
            *h != raiz && ffi::estado_ventana(*h).habilitada && puede_bloquear(&class_of(*h))
        })
    })
}

/// `modal: <class> 0x<hwnd>`: how an answer names a dialog that blocks a window.
/// Every action that refuses a blocked window says it this way, so a caller can
/// read the dialog to close without parsing prose.
fn nombre_de(modal: Hwnd) -> String {
    format!("modal: {} 0x{modal:X}", class_of(modal))
}

/// The same name, for the dialog that blocks `raiz`, read from the window.
pub(crate) fn nombre_del_bloqueo(raiz: Hwnd) -> String {
    bloqueante_de(raiz).map_or_else(|| "modal: ?".to_string(), nombre_de)
}

/// Fails when the top-level window of `ventana` is disabled by a modal dialog.
/// `para` completes the hint: "…before {para}".
pub(crate) fn exigir_habilitada(ventana: Hwnd, para: &str) -> Result<()> {
    let raiz = ffi::get_ancestor(ventana, GA_ROOT);
    if !ffi::es_ventana(raiz) || ffi::estado_ventana(raiz).habilitada {
        return Ok(());
    }
    Err(Error::Gui(format!(
        "{} is disabled: a modal dialog is open on top of it ({}). Close it (gui_actuar accion:cerrar) before {para}",
        class_of(raiz),
        nombre_del_bloqueo(raiz)
    )))
}

/// The modal blocking the process, if any: a VCL modal can be a child window of
/// the form it disables, and `EnumWindows` would not list it.
pub(crate) fn modal_de(pids: &[u32]) -> Option<Hwnd> {
    pids.iter()
        .flat_map(|p| enum_windows_de(*p))
        .filter(|h| visible(*h))
        .find_map(bloqueante_de)
}

/// What an action does with the dialog that blocked the window it wanted.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum AvisoDemo {
    /// PVsyst's DEMO notice: press this caption to dismiss it.
    Cerrar { boton: Rotulo<'static> },
    /// Any other dialog, or a notice this server cannot dismiss: nothing is
    /// pressed, so a question is still answered by a person and the action
    /// fails as it did before.
    OtroDialogo,
}

/// Whether the modal of class `clase`, whose visible captions are `botones`, is
/// PVsyst's DEMO notice, and the button that dismisses it.
///
/// Only the notice counts: it informs and asks nothing. The caption comes from
/// the language tables, so a notice whose «Cerrar» is not among its buttons — a
/// GUI in a language whose caption was never measured — presses nothing and the
/// action fails as before.
pub(crate) fn aviso_demo(clase: &str, botones: &[&str], idioma: Option<IdiomaGui>) -> AvisoDemo {
    if clase != CLASE_AVISO_DEMO {
        return AvisoDemo::OtroDialogo;
    }
    let Some(idioma) = idioma else {
        return AvisoDemo::OtroDialogo;
    };
    if botones.contains(&CERRAR.en_idioma(idioma)) {
        AvisoDemo::Cerrar { boton: CERRAR }
    } else {
        AvisoDemo::OtroDialogo
    }
}

/// Note of a DEMO notice an action dismissed: the class, the hwnd and the
/// caption pressed, so the click does not pass for the caller's own doing.
const AVISO_DEMO_CERRADO: Texto = Texto {
    es: "PVsyst tenía abierto su aviso «Modo DEMO» ({clase} 0x{hwnd}): es solo informativo y bloqueaba las ventanas, así que se cerró con «{boton}» y la acción se repitió una vez",
    en: "PVsyst had its DEMO notice open ({clase} 0x{hwnd}): it only informs and it blocked the windows, so it was closed with «{boton}» and the action was tried once more",
};

/// Dismisses PVsyst's DEMO notice when it is the dialog that made `fallo` fail,
/// and names it. `None` when the failure is another one, when the dialog is not
/// the notice, or when the notice is still there afterwards: the caller keeps
/// its own error, exactly as before.
pub(crate) fn despejar_aviso_demo(fallo: &str, idioma: Idioma) -> Option<String> {
    let dialogo = modal_de(&pids_pvsyst())?;
    // The refusal of the guard names this very dialog. Another failure with the
    // notice on screen (a mistyped window, a missing field) keeps its error.
    if !fallo.contains(nombre_de(dialogo).as_str()) {
        return None;
    }
    let idioma_gui = idioma_gui::en_ejecucion()?;
    let clase = class_of(dialogo);
    let textos: Vec<String> = hijos_visibles(dialogo).map(text_of).collect();
    let botones: Vec<&str> = textos.iter().map(String::as_str).collect();
    let AvisoDemo::Cerrar { boton } = aviso_demo(&clase, &botones, Some(idioma_gui)) else {
        return None;
    };
    if let Err(error) = Formulario::con_idioma(dialogo, idioma).pulsar(boton) {
        tracing::warn!(%error, "no se pudo pulsar el cierre del aviso DEMO");
        return None;
    }
    if !esperar_cierre(dialogo, ESPERA_CIERRE) {
        tracing::warn!(
            clase = %clase,
            hwnd = format!("0x{dialogo:X}"),
            "el aviso DEMO sigue abierto tras pulsar su botón de cierre"
        );
        return None;
    }
    Some(AVISO_DEMO_CERRADO.texto_con(
        idioma,
        &[
            ("clase", &clase),
            ("hwnd", &format!("{dialogo:X}")),
            ("boton", boton.en_idioma(idioma_gui)),
        ],
    ))
}

/// Adds `nota` to the answer's `advertencias`, the key the agent view reads as
/// `avisos`, keeping whatever the action itself warned about.
pub(crate) fn anotar(respuesta: &mut Value, nota: Option<String>) {
    let Some(nota) = nota else {
        return;
    };
    let Some(mapa) = respuesta.as_object_mut() else {
        tracing::warn!(
            clase = CLASE_AVISO_DEMO,
            "la respuesta de la acción no es un objeto: no se pudo anotar que el aviso DEMO se cerró"
        );
        return;
    };
    let mut avisos = match mapa.remove("advertencias") {
        Some(Value::Array(avisos)) => avisos,
        Some(otro) => vec![otro],
        None => Vec::new(),
    };
    avisos.push(Value::String(nota));
    mapa.insert("advertencias".into(), Value::Array(avisos));
}

/// Adds the blocking modal when the enumeration did not list it, without
/// duplicating one that is already there.
pub(crate) fn agregar_modal(out: &mut Vec<Value>, modal: Option<Value>) {
    let Some(mut modal) = modal else {
        return;
    };
    let hwnd = modal.get("hwnd").cloned().unwrap_or(Value::Null);
    if out.iter().any(|w| w.get("hwnd") == Some(&hwnd)) {
        return;
    }
    if let Some(mapa) = modal.as_object_mut() {
        mapa.insert("modal".into(), json!(true));
        mapa.insert("child".into(), json!(true));
    }
    out.push(modal);
}

#[cfg(test)]
#[path = "../../../tests/unit/adapters/gui/modal.rs"]
mod tests;
