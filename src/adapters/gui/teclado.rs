//! Keyboard shortcuts, modifier combination and virtual key mapping.

use super::escritorio;
use super::ffi;
use super::ventanas::{resolve, ventanas_de};
use crate::args::req_str;
use crate::error::{Error, Result};
use serde_json::{json, Value};
use std::time::Duration;

const VK_CONTROL: u8 = 0x11;
const VK_SHIFT: u8 = 0x10;
const VK_MENU: u8 = 0x12;
/// Lets Windows move the focus to the target window before typing.
const FOCO: Duration = Duration::from_millis(80);

fn vk_for(key: &str) -> Option<u8> {
    let k = key.trim().to_uppercase();
    match k.as_str() {
        "F1" => Some(0x70),
        "F2" => Some(0x71),
        "F3" => Some(0x72),
        "F4" => Some(0x73),
        "F5" => Some(0x74),
        "F6" => Some(0x75),
        "F7" => Some(0x76),
        "F8" => Some(0x77),
        "F9" => Some(0x78),
        "F10" => Some(0x79),
        "F11" => Some(0x7A),
        "F12" => Some(0x7B),
        "ENTER" | "RETURN" => Some(0x0D),
        "ESC" | "ESCAPE" => Some(0x1B),
        "TAB" => Some(0x09),
        "SPACE" => Some(0x20),
        "LEFT" => Some(0x25),
        "UP" => Some(0x26),
        "RIGHT" => Some(0x27),
        "DOWN" => Some(0x28),
        _ => match k.as_bytes() {
            &[c] if c.is_ascii_uppercase() || c.is_ascii_digit() => Some(c),
            _ => None,
        },
    }
}

pub(crate) fn teclas(paths: &crate::config::Paths, args: &Value) -> Result<Value> {
    let _ = paths;
    let scoped = ventanas_de(args)?;
    let reference = args.get("ventana").and_then(|v| v.as_str());
    let h = match reference {
        Some(r) => {
            resolve(&scoped, r).ok_or_else(|| Error::Gui(format!("Window '{r}' not found")))?
        }
        None => ffi::primer_plano(),
    };
    if h == 0 {
        return Err(Error::Gui("No hay una ventana en primer plano".into()));
    }
    let combo = req_str(args, &["teclas"], "e.g. \"F1\" or \"CTRL+S\"")?.to_uppercase();
    let parts: Vec<&str> = combo.split('+').map(str::trim).collect();
    let mut mods = Vec::new();
    let mut principales = Vec::new();
    for p in &parts {
        match *p {
            "CTRL" | "CONTROL" => mods.push(VK_CONTROL),
            "SHIFT" => mods.push(VK_SHIFT),
            "ALT" => mods.push(VK_MENU),
            other => principales.push(vk_for(other).ok_or_else(|| {
                Error::Gui(format!("Tecla no reconocida '{other}' en '{combo}'"))
            })?),
        }
    }
    let &[vk] = principales.as_slice() else {
        return Err(Error::Gui(format!(
            "'{combo}' necesita exactamente una tecla principal (p. ej. CTRL+S)"
        )));
    };
    if escritorio::es_aislado() && mods.iter().any(|m| *m != VK_MENU) {
        return Err(Error::Gui(
            "En el escritorio aislado solo se admite Alt como modificador: Ctrl y Mayús no se pueden enviar como mensajes".into(),
        ));
    }
    ffi::traer_al_frente(h);
    std::thread::sleep(FOCO);
    ffi::pulsar_combinacion(vk, &mods);
    Ok(json!({
        "sent": combo,
        "hwnd": format!("0x{:X}", h),
        "via": if escritorio::es_aislado() { "mensajes" } else { "keybd_event" },
    }))
}
