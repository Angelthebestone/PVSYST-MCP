//! Keyboard shortcuts, modifier combination and virtual key mapping.

use super::combinacion::{parsear_combinacion, VK_MENU};
use super::escritorio;
use super::ffi;
use super::ventanas::{class_of, rect_of, resolve, ventanas_de, visible};
use crate::args::req_str;
use crate::error::{Error, Result};
use serde_json::{json, Value};
use std::time::Duration;

/// Lets Windows move the focus to the target window before typing.
const FOCO: Duration = Duration::from_millis(80);

/// The keys go to whoever has the focus, so a control target must really have
/// it: an arrow sent to the wrong control changes a combo instead of a list. On the
/// desktop of the user nothing is sent without that guarantee, because the keys could
/// reach another application; on the isolated desktop only PVsyst's own windows can
/// receive them, so a focus that did not move is reported (`false`) and the keys go on.
fn dar_foco_al_control(h: ffi::Hwnd) -> Result<bool> {
    let destino = if escritorio::es_aislado() {
        Destino::Aislado
    } else {
        Destino::Usuario
    };
    resolver_foco(ffi::dar_foco(h), destino, h)
}

/// The desktop the keys are sent on.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Destino {
    Usuario,
    Aislado,
}

/// What a focus attempt (`logrado`) means for sending the keys: `Ok(foco dado)` or an error.
fn resolver_foco(logrado: bool, destino: Destino, h: ffi::Hwnd) -> Result<bool> {
    if logrado {
        return Ok(true);
    }
    if destino == Destino::Aislado {
        tracing::warn!(
            control = h,
            "the control did not take the focus on the isolated desktop: the keys are sent anyway"
        );
        return Ok(false);
    }
    Err(Error::Gui(format!(
        "Could not give the focus to control 0x{h:X}: keys are not sent without that guarantee. Target the host window or use accion:clic_fisico on the control"
    )))
}

/// Frontmost visible window of the target process. Never the foreground window of
/// the desktop: it may belong to another application, which would get the keys.
fn ventana_mas_al_frente(alcance: &[ffi::Hwnd]) -> Result<ffi::Hwnd> {
    alcance
        .iter()
        .copied()
        .find(|h| visible(*h) && rect_of(*h).right > rect_of(*h).left)
        .ok_or_else(|| Error::Gui("The process has no visible window".into()))
}

/// `keybd_event` goes to the foreground window, so the target must really be it.
/// The isolated desktop posts messages to its own windows and has no foreground to check.
fn exigir_primer_plano(raiz: ffi::Hwnd) -> Result<()> {
    if escritorio::es_aislado() || ffi::primer_plano() == raiz {
        return Ok(());
    }
    Err(Error::Gui(format!(
        "PVsyst did not take the foreground ({}): keys are not sent to another application",
        class_of(raiz)
    )))
}

pub(crate) fn teclas(args: &Value) -> Result<Value> {
    let combo = req_str(args, &["teclas"], "e.g. \"F1\" or \"CTRL+S\"")?.to_uppercase();
    let (mods, vk) = parsear_combinacion(&combo)?;
    let scoped = ventanas_de(args)?;
    let reference = args.get("ventana").and_then(|v| v.as_str());
    let h = match reference {
        Some(r) => {
            resolve(&scoped, r).ok_or_else(|| Error::Gui(format!("Window '{r}' not found")))?
        }
        None => ventana_mas_al_frente(&scoped)?,
    };
    if escritorio::es_aislado() && mods.iter().any(|m| *m != VK_MENU) {
        return Err(Error::Gui(
            "On the isolated desktop only Alt is accepted as a modifier: Ctrl and Shift cannot be sent as messages".into(),
        ));
    }
    let raiz = ffi::get_ancestor(h, ffi::GA_ROOT);
    exigir_objetivo_habilitado(ffi::estado_ventana(raiz).habilitada, &class_of(raiz))?;
    ffi::traer_al_frente(raiz);
    std::thread::sleep(FOCO);
    exigir_primer_plano(raiz)?;
    let foco = raiz != h && dar_foco_al_control(h)?;
    let foco_actual = ffi::foco_del_hilo(ffi::hilo_de(raiz));
    let raiz_del_foco = (foco_actual != 0).then(|| ffi::get_ancestor(foco_actual, ffi::GA_ROOT));
    exigir_foco_en_la_ventana(foco_actual, raiz, raiz_del_foco)?;
    ffi::pulsar_combinacion(vk, &mods);
    Ok(json!({
        "sent": combo,
        "hwnd": format!("0x{:X}", h),
        "foco": foco,
        "aviso": aviso_de_foco(foco, if raiz == h { Objetivo::Ventana } else { Objetivo::Control }),
        "via": if escritorio::es_aislado() { "mensajes" } else { "keybd_event" },
    }))
}

/// A target disabled by a modal dialog would receive nothing: the keys would go
/// to the dialog instead, which is how a blind RETURN ended PVsyst.
fn exigir_objetivo_habilitado(habilitada: bool, clase: &str) -> Result<()> {
    if habilitada {
        return Ok(());
    }
    Err(Error::Gui(format!(
        "'{clase}' is disabled by a modal dialog: the keys would go to the dialog, not to the window. Close it (gui_actuar accion:cerrar) or send 'teclas' to the dialog class and retry"
    )))
}

/// Keys go to whichever control has the focus: it must live in the window the
/// caller named, or another window would receive them.
fn exigir_foco_en_la_ventana(
    foco: ffi::Hwnd,
    raiz: ffi::Hwnd,
    raiz_del_foco: Option<ffi::Hwnd>,
) -> Result<()> {
    if foco == 0 || raiz_del_foco == Some(raiz) {
        return Ok(());
    }
    Err(Error::Gui(format!(
        "The keyboard focus is in another window (0x{foco:X}), not in 0x{raiz:X}: the keys are not sent. Bring the target to the front with accion:activar and retry"
    )))
}

/// What the caller named: a whole window or a control inside it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Objetivo {
    Ventana,
    Control,
}

/// What `foco:false` means, so a successful call is not read as a guaranteed focus.
fn aviso_de_foco(foco: bool, objetivo: Objetivo) -> Option<&'static str> {
    if foco {
        return None;
    }
    Some(match objetivo {
        Objetivo::Ventana => {
            "The target is the window: the keys reach the control that already has the focus inside it"
        }
        Objetivo::Control => {
            "The control did not take the focus on the isolated desktop: the keys were sent anyway"
        }
    })
}

#[cfg(test)]
#[path = "../../../tests/unit/adapters/gui/teclado.rs"]
mod tests;
