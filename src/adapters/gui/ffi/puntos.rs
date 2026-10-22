//! Screen points: which window sits under one, and a mouse click posted to a
//! window at a point of its client area. Controls without a window of their
//! own (VCL speed buttons, grid cells, tree nodes) are reached this way.

use super::ventanas::{es_ventana, post_message, Mensaje, WM_LBUTTONDOWN, WM_LBUTTONUP};
use super::Hwnd;

const WM_MOUSEMOVE: u32 = 0x0200;
const MK_LBUTTON: usize = 0x0001;
/// `ChildWindowFromPointEx` flag: hidden children never hit.
const CWP_SKIPINVISIBLE: u32 = 0x0001;

/// A point in pixels, laid out as the Win32 `POINT`.
#[repr(C)]
#[derive(Clone, Copy, Default)]
pub(crate) struct Punto {
    pub(crate) x: i32,
    pub(crate) y: i32,
}

#[link(name = "user32")]
extern "system" {
    fn ScreenToClient(h: Hwnd, p: *mut Punto) -> i32;
    fn ChildWindowFromPointEx(h: Hwnd, p: Punto, flags: u32) -> Hwnd;
    fn WindowFromPoint(p: Punto) -> Hwnd;
}

/// `(x, y)` screen pixels as client pixels of `h`; `None` when Windows cannot
/// convert (a destroyed window).
pub(crate) fn a_cliente(h: Hwnd, (x, y): (i32, i32)) -> Option<(i32, i32)> {
    if !es_ventana(h) {
        return None;
    }
    let mut p = Punto { x, y };
    // SAFETY: `p` es una estructura POINT local en la pila donde `ScreenToClient` escribe (x, y).
    let convertido = unsafe { ScreenToClient(h, &mut p) } != 0;
    convertido.then_some((p.x, p.y))
}

fn hijo_directo(padre: Hwnd, punto: (i32, i32)) -> Option<Hwnd> {
    let (x, y) = a_cliente(padre, punto)?;
    // SAFETY: Consulta de solo lectura; `Punto` se pasa por valor como pide la API.
    let hijo = unsafe { ChildWindowFromPointEx(padre, Punto { x, y }, CWP_SKIPINVISIBLE) };
    (hijo != 0 && hijo != padre).then_some(hijo)
}

/// Deepest visible window under the screen point `punto`, searching inside
/// `raiz`; `raiz` itself when no child contains it. O(depth).
pub(crate) fn control_en_punto(raiz: Hwnd, punto: (i32, i32)) -> Hwnd {
    std::iter::successors(Some(raiz), |h| hijo_directo(*h, punto))
        .last()
        .unwrap_or(raiz)
}

/// Window under the screen point, whatever process owns it; 0 when none.
pub(crate) fn ventana_en(x: i32, y: i32) -> Hwnd {
    // SAFETY: Consulta de solo lectura; `Punto` se pasa por valor como pide la API.
    unsafe { WindowFromPoint(Punto { x, y }) }
}

/// A client point packed as the `lParam` of a mouse message.
fn empaquetar((x, y): (i32, i32)) -> isize {
    (isize::from(x as i16) & 0xFFFF) | ((isize::from(y as i16) & 0xFFFF) << 16)
}

/// Posts move, left-down and left-up at `(x, y)` in the client coordinates `h`
/// itself understands (a DPI-unaware window reads its 96-DPI layout). The
/// release is always sent once the press was queued, so a control never stays
/// pressed. `false` when Windows refuses to queue a message.
pub(crate) fn clic_en_cliente(h: Hwnd, punto: (i32, i32)) -> bool {
    let lparam = empaquetar(punto);
    if !post_message(h, Mensaje::nuevo(WM_MOUSEMOVE, 0, lparam))
        || !post_message(h, Mensaje::nuevo(WM_LBUTTONDOWN, MK_LBUTTON, lparam))
    {
        return false;
    }
    post_message(h, Mensaje::nuevo(WM_LBUTTONUP, 0, lparam))
}

/// Why a click on a screen point was not delivered.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum FalloClic {
    /// Windows could not turn the screen point into a point of the window.
    SinConversion,
    /// Windows refused to queue a mouse message.
    SinEnvio,
}

/// Clicks the screen point `punto` on window `h` by posting mouse messages.
/// `ScreenToClient` already answers in the coordinates `h` itself uses (the
/// 96-DPI layout for a DPI-unaware window such as PVsyst), so no scaling.
pub(crate) fn clic_en_pantalla(h: Hwnd, punto: (i32, i32)) -> Result<(), FalloClic> {
    let cliente = a_cliente(h, punto).ok_or(FalloClic::SinConversion)?;
    if clic_en_cliente(h, cliente) {
        Ok(())
    } else {
        Err(FalloClic::SinEnvio)
    }
}
