//! Keyboard shortcuts, modifier combination and virtual key mapping.

use super::ffi::{self, WM_KEYDOWN, WM_KEYUP};
use super::ventanas::{resolve, ventanas_de};
use crate::args::req_str;
use crate::error::{Error, Result};
use serde_json::{json, Value};

const VK_CONTROL: u32 = 0x11;
const VK_SHIFT: u32 = 0x10;
const VK_MENU: u32 = 0x12;

fn vk_for(key: &str) -> Option<u32> {
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
        _ => {
            let bytes = k.as_bytes();
            if bytes.len() == 1 {
                let c = bytes[0];
                if c.is_ascii_uppercase() || c.is_ascii_digit() {
                    return Some(u32::from(c));
                }
            }
            None
        }
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
        // SAFETY: Sin argumentos; devuelve 0 si no hay ventana en primer plano.
        None => unsafe { ffi::GetForegroundWindow() },
    };
    let combo = req_str(args, &["teclas"], "e.g. \"F1\" or \"CTRL+S\"")?.to_uppercase();
    let parts: Vec<&str> = combo.split('+').map(str::trim).collect();
    let mut mods = Vec::new();
    let mut main = None;
    for p in &parts {
        match *p {
            "CTRL" | "CONTROL" => mods.push(VK_CONTROL),
            "SHIFT" => mods.push(VK_SHIFT),
            "ALT" => mods.push(VK_MENU),
            other => main = vk_for(other),
        }
    }
    let vk = main.ok_or_else(|| Error::Gui(format!("Unrecognized key in '{combo}'")))?;
    // SAFETY: `PostMessageW` encola mensajes con enteros; un HWND inválido solo hace fallar la llamada.
    unsafe {
        for m in &mods {
            ffi::PostMessageW(h, WM_KEYDOWN, *m as usize, 0);
        }
        ffi::PostMessageW(h, WM_KEYDOWN, vk as usize, 0);
        ffi::PostMessageW(h, WM_KEYUP, vk as usize, 0);
        for m in mods.iter().rev() {
            ffi::PostMessageW(h, WM_KEYUP, *m as usize, 0);
        }
    }
    Ok(json!({ "sent": combo, "hwnd": format!("0x{:X}", h) }))
}
