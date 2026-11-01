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
            "el control no tomó el foco en el escritorio aislado: se envían las teclas igualmente"
        );
        return Ok(false);
    }
    Err(Error::Gui(format!(
        "No se pudo dar el foco al control 0x{h:X}: las teclas no se envían sin esa garantía. Usa la ventana contenedora o accion:clic_fisico sobre el control"
    )))
}

/// Frontmost visible window of the target process. Never the foreground window of
/// the desktop: it may belong to another application, which would get the keys.
fn ventana_mas_al_frente(alcance: &[ffi::Hwnd]) -> Result<ffi::Hwnd> {
    alcance
        .iter()
        .copied()
        .find(|h| visible(*h) && rect_of(*h).right > rect_of(*h).left)
        .ok_or_else(|| Error::Gui("El proceso no tiene ninguna ventana visible".into()))
}

/// `keybd_event` goes to the foreground window, so the target must really be it.
/// The isolated desktop posts messages to its own windows and has no foreground to check.
fn exigir_primer_plano(raiz: ffi::Hwnd) -> Result<()> {
    if escritorio::es_aislado() || ffi::primer_plano() == raiz {
        return Ok(());
    }
    Err(Error::Gui(format!(
        "PVsyst no tomó el primer plano ({}): no se envían teclas a otra aplicación",
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
            "En el escritorio aislado solo se admite Alt como modificador: Ctrl y Mayús no se pueden enviar como mensajes".into(),
        ));
    }
    let raiz = ffi::get_ancestor(h, ffi::GA_ROOT);
    ffi::traer_al_frente(raiz);
    std::thread::sleep(FOCO);
    exigir_primer_plano(raiz)?;
    let foco = raiz != h && dar_foco_al_control(h)?;
    ffi::pulsar_combinacion(vk, &mods);
    Ok(json!({
        "sent": combo,
        "hwnd": format!("0x{:X}", h),
        "foco": foco,
        "via": if escritorio::es_aislado() { "mensajes" } else { "keybd_event" },
    }))
}

#[cfg(test)]
#[path = "../../../tests/unit/adapters/gui/teclado.rs"]
mod tests;
