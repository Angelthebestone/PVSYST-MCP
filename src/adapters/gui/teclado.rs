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

/// First virtual key of F1..F12; `VK_F1 + n` is F(n+1).
const VK_F1: u8 = 0x70;
const TECLAS_DE_FUNCION: u8 = 12;

/// Named keys the tool sends, each with its spellings. HOME/END/PGUP/PGDN move
/// the selection of lists and trees; nothing here edits or deletes.
const TECLAS_NOMBRADAS: &[(&[&str], u8)] = &[
    (&["ENTER", "RETURN"], 0x0D),
    (&["ESC", "ESCAPE"], 0x1B),
    (&["TAB"], 0x09),
    (&["SPACE"], 0x20),
    (&["PGUP", "PAGEUP"], 0x21),
    (&["PGDN", "PAGEDOWN"], 0x22),
    (&["END"], 0x23),
    (&["HOME"], 0x24),
    (&["LEFT"], 0x25),
    (&["UP"], 0x26),
    (&["RIGHT"], 0x27),
    (&["DOWN"], 0x28),
];

/// Keys that are refused: in PVsyst lists they can delete or insert items, and
/// deleting is outside what this tool does.
const TECLAS_PROHIBIDAS: &[&str] = &["DELETE", "DEL", "SUPR", "INSERT", "INS"];

/// Number after `F` when `key` is `F1`..`F12`.
fn numero_de_funcion(key: &str) -> Option<u8> {
    let n: u8 = key.strip_prefix('F')?.parse().ok()?;
    (1..=TECLAS_DE_FUNCION).contains(&n).then_some(n)
}

fn vk_for(key: &str) -> Option<u8> {
    let k = key.trim().to_uppercase();
    if let Some(n) = numero_de_funcion(&k) {
        return Some(VK_F1 + n - 1);
    }
    if let Some((_, vk)) = TECLAS_NOMBRADAS
        .iter()
        .find(|(nombres, _)| nombres.contains(&k.as_str()))
    {
        return Some(*vk);
    }
    match k.as_bytes() {
        &[c] if c.is_ascii_uppercase() || c.is_ascii_digit() => Some(c),
        _ => None,
    }
}

/// Splits `CTRL+S` into its modifier codes and its one principal key. An
/// unknown token or a combo without exactly one principal key is an error.
fn parsear_combinacion(combo: &str) -> Result<(Vec<u8>, u8)> {
    let mut mods = Vec::new();
    let mut principales = Vec::new();
    for p in combo.split('+').map(str::trim) {
        match p {
            "CTRL" | "CONTROL" => mods.push(VK_CONTROL),
            "SHIFT" => mods.push(VK_SHIFT),
            "ALT" => mods.push(VK_MENU),
            other if TECLAS_PROHIBIDAS.contains(&other.to_uppercase().as_str()) => {
                return Err(Error::Gui(format!(
                    "La tecla '{other}' no se envía: en las listas de PVsyst puede eliminar o insertar elementos"
                )));
            }
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
    Ok((mods, vk))
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
    let (mods, vk) = parsear_combinacion(&combo)?;
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

#[cfg(test)]
#[path = "../../../tests/unit/adapters/gui/teclado.rs"]
mod tests;
