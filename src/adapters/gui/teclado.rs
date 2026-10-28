//! Keyboard shortcuts, modifier combination and virtual key mapping.

use super::escritorio;
use super::ffi;
use super::ventanas::{class_of, rect_of, resolve, ventanas_de, visible};
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

/// The keys go to whoever has the focus, so a control target must really have
/// it: an arrow sent to the wrong control changes a combo instead of a list.
/// Never sent without that guarantee.
fn dar_foco_al_control(h: ffi::Hwnd) -> Result<()> {
    if !escritorio::es_aislado() && ffi::dar_foco(h) {
        return Ok(());
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
    if raiz != h {
        dar_foco_al_control(h)?;
    }
    ffi::pulsar_combinacion(vk, &mods);
    Ok(json!({
        "sent": combo,
        "hwnd": format!("0x{:X}", h),
        "foco": raiz != h,
        "via": if escritorio::es_aislado() { "mensajes" } else { "keybd_event" },
    }))
}

#[cfg(test)]
#[path = "../../../tests/unit/adapters/gui/teclado.rs"]
mod tests;
