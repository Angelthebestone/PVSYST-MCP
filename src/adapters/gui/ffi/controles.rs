//! Control-level Win32 calls: client coordinates, DPI, text in and out of edit
//! controls, tab pages and the keyboard Tab key.

use super::puntos::Punto;
use super::Hwnd;

const WM_GETTEXT: u32 = 0x000D;
const WM_CHAR: u32 = 0x0102;
const EM_SETSEL: u32 = 0x00B1;
const CB_FINDSTRING: u32 = 0x014C;
const CB_SETCURSEL: u32 = 0x014E;
const LB_FINDSTRING: u32 = 0x018F;
const LB_SETCURSEL: u32 = 0x0186;
const LB_SETSEL: u32 = 0x0185;
const GWL_STYLE: i32 = -16;
/// `LBS_MULTIPLESEL | LBS_EXTENDEDSEL`: the list keeps several selected items.
const ESTILO_MULTISELECCION: isize = 0x0008 | 0x0800;
pub(crate) const WM_COMMAND: u32 = 0x0111;
const CBN_SELCHANGE: usize = 1;
/// `CB_FINDSTRING` result when no item matches.
const CB_ERR: isize = -1;
/// `CB_FINDSTRING` start index meaning "search the whole list".
const DESDE_EL_PRINCIPIO: usize = usize::MAX;
const TCM_SETCURFOCUS: u32 = 0x1330;
const VK_TAB: u8 = 0x09;
const VK_ALT: u8 = 0x12;
const KEYEVENTF_KEYUP: u32 = 0x0002;
/// DPI at which Windows lays out an unscaled window.
pub(crate) const DPI_BASE: u32 = 96;
/// `DPI_AWARENESS_CONTEXT_PER_MONITOR_AWARE_V2`: rects, cursor and clicks use physical pixels.
const DPI_PER_MONITOR_V2: isize = -4;
/// `MonitorFromWindow` flag: the monitor nearest to the window when it is on none.
const MONITOR_DEFAULTTONEAREST: u32 = 2;
/// `GetDpiForMonitor` kind: the DPI the user chose in Settings, not the raw one.
const MDT_EFFECTIVE_DPI: u32 = 0;
/// `DPI_AWARENESS_UNAWARE`: Windows virtualises the process to a 96-DPI layout.
const DPI_AWARENESS_UNAWARE: i32 = 0;
/// Longest text read back from a PVsyst edit control.
const LARGO_TEXTO: usize = 1024;

#[link(name = "user32")]
extern "system" {
    fn ClientToScreen(h: Hwnd, p: *mut Punto) -> i32;
    fn GetDpiForWindow(h: Hwnd) -> u32;
    fn GetWindowDpiAwarenessContext(h: Hwnd) -> isize;
    fn GetAwarenessFromDpiAwarenessContext(contexto: isize) -> i32;
    fn MonitorFromWindow(h: Hwnd, flags: u32) -> isize;
    fn SetProcessDpiAwarenessContext(contexto: isize) -> i32;
    fn SendMessageW(h: Hwnd, msg: u32, w: usize, l: isize) -> isize;
    fn GetParent(h: Hwnd) -> Hwnd;
    fn GetDlgCtrlID(h: Hwnd) -> i32;
    fn GetWindowLongPtrW(h: Hwnd, indice: i32) -> isize;
    fn keybd_event(vk: u8, scan: u8, flags: u32, extra: usize);
}

#[link(name = "shcore")]
extern "system" {
    fn GetDpiForMonitor(monitor: isize, tipo: u32, x: *mut u32, y: *mut u32) -> i32;
}

/// Screen position of the client-area origin of `h`.
pub(crate) fn origen_cliente(h: Hwnd) -> (i32, i32) {
    let mut p = Punto::default();
    // SAFETY: `p` es una local viva; la API solo escribe en ella.
    unsafe {
        ClientToScreen(h, &mut p);
    }
    (p.x, p.y)
}

/// Whether `h` belongs to a DPI-unaware process: Windows virtualises its
/// coordinates and bitmaps to a 96-DPI layout.
pub(crate) fn ventana_ignora_dpi(h: Hwnd) -> bool {
    // SAFETY: Consultas de solo lectura sobre un HWND; devuelven valores sueltos.
    unsafe {
        GetAwarenessFromDpiAwarenessContext(GetWindowDpiAwarenessContext(h))
            == DPI_AWARENESS_UNAWARE
    }
}

/// Makes this process Per-Monitor-DPI-aware (v2) so every rect, cursor position
/// and click is in physical pixels. Without it Windows hands a DPI-unaware
/// process virtualised coordinates and `GetDpiForWindow` answers 96 at any
/// scale. Call once at start-up, before any window or UI Automation call. A
/// later call is a no-op: Windows refuses to change an awareness already set.
pub(crate) fn declarar_dpi_por_monitor() {
    // SAFETY: Sin punteros; solo fija el modo de DPI del proceso actual.
    unsafe {
        SetProcessDpiAwarenessContext(DPI_PER_MONITOR_V2);
    }
}

/// Scale of the monitor `h` is on, in DPI (96 = 100 %); `DPI_BASE` when Windows
/// does not say. The monitor is asked, not the window: a DPI-unaware PVsyst
/// window reports 96 for itself but is stretched to the monitor's scale, so its
/// 96-DPI layout positions map to physical pixels by the monitor's DPI.
pub(crate) fn dpi_de(h: Hwnd) -> u32 {
    let (mut x, mut y) = (0u32, 0u32);
    // SAFETY: `x` e `y` son locales vivas que la API solo escribe; el monitor lo
    // devuelve Windows para un HWND (0 si no es válido y la llamada falla).
    let de_monitor = unsafe {
        let monitor = MonitorFromWindow(h, MONITOR_DEFAULTTONEAREST);
        monitor != 0 && GetDpiForMonitor(monitor, MDT_EFFECTIVE_DPI, &mut x, &mut y) == 0
    };
    if de_monitor && x != 0 {
        return x;
    }
    // SAFETY: Consulta de solo lectura sobre un HWND; devuelve 0 si no es válido.
    match unsafe { GetDpiForWindow(h) } {
        0 => DPI_BASE,
        dpi => dpi,
    }
}

/// Factor from capture pixels to physical pixels. `PrintWindow` renders a
/// DPI-unaware window (PVsyst) at its 96-DPI layout inside a bitmap as large
/// as the physical window, so its text sits at `1/scale` of the real position.
pub(crate) fn escala_de_captura(h: Hwnd) -> f64 {
    if ventana_ignora_dpi(h) {
        f64::from(dpi_de(h)) / f64::from(DPI_BASE)
    } else {
        1.0
    }
}

/// `v` laid out at 96 DPI, scaled to `dpi`. O(1).
pub(crate) fn escalar_a_dpi(v: i32, dpi: u32) -> i32 {
    let base = i64::from(DPI_BASE);
    let escalado = i64::from(v) * i64::from(dpi) / base;
    i32::try_from(escalado).unwrap_or(v)
}

/// Text of a control in another process (`WM_GETTEXT` is marshalled by Windows).
pub(crate) fn texto_control(h: Hwnd) -> String {
    let mut buf = vec![0u16; LARGO_TEXTO];
    // SAFETY: `buf` vive durante la llamada síncrona y su longitud se pasa en wparam.
    let n = unsafe { SendMessageW(h, WM_GETTEXT, buf.len(), buf.as_mut_ptr() as isize) };
    let n = usize::try_from(n).unwrap_or(0).min(buf.len());
    String::from_utf16_lossy(&buf[..n])
}

/// Replaces the content of a focused edit control as if typed, so the owning
/// form sees every change (VCL recalculates on each keystroke).
pub(crate) fn teclear_en(h: Hwnd, texto: &str) {
    // SAFETY: Mensajes con enteros sobre el HWND recibido; sin punteros.
    unsafe {
        SendMessageW(h, EM_SETSEL, 0, -1);
        for unidad in texto.encode_utf16() {
            SendMessageW(h, WM_CHAR, usize::from(unidad), 0);
        }
    }
}

/// Shows tab page `indice` of a page control.
pub(crate) fn mostrar_pagina(h: Hwnd, indice: usize) {
    // SAFETY: Mensaje con enteros sobre el HWND recibido.
    unsafe {
        SendMessageW(h, TCM_SETCURFOCUS, indice, 0);
    }
}

/// Presses Tab in the foreground window, which commits the focused field.
pub(crate) fn pulsar_tabulador() {
    if super::aislado() {
        super::tecla_emulada(VK_TAB, false);
        return;
    }
    // SAFETY: Evento de teclado sin punteros.
    unsafe {
        keybd_event(VK_TAB, 0, 0, 0);
        keybd_event(VK_TAB, 0, KEYEVENTF_KEYUP, 0);
    }
}

/// Sends a key combination to the foreground window as real keyboard input,
/// which VCL forms honour (posted `WM_KEY*` messages are usually ignored). On
/// a private desktop there is no real input, so the key is posted instead and
/// only Alt is kept (see `tecla_emulada`).
pub(crate) fn pulsar_combinacion(vk: u8, modificadores: &[u8]) {
    if super::aislado() {
        super::tecla_emulada(vk, modificadores.contains(&VK_ALT));
        return;
    }
    // SAFETY: Eventos de teclado sin punteros.
    unsafe {
        for m in modificadores {
            keybd_event(*m, 0, 0, 0);
        }
        keybd_event(vk, 0, 0, 0);
        keybd_event(vk, 0, KEYEVENTF_KEYUP, 0);
        for m in modificadores.iter().rev() {
            keybd_event(*m, 0, KEYEVENTF_KEYUP, 0);
        }
    }
}

/// Window messages of a list control: find an item, select it.
struct MensajesLista {
    buscar: u32,
    elegir: u32,
}

const COMBO: MensajesLista = MensajesLista {
    buscar: CB_FINDSTRING,
    elegir: CB_SETCURSEL,
};
const LISTA: MensajesLista = MensajesLista {
    buscar: LB_FINDSTRING,
    elegir: LB_SETCURSEL,
};

/// Whether the list box `h` keeps several items selected at once.
fn es_multiseleccion(h: Hwnd) -> bool {
    // SAFETY: Consulta de solo lectura del estilo de un HWND recibido.
    unsafe { GetWindowLongPtrW(h, GWL_STYLE) & ESTILO_MULTISELECCION != 0 }
}

/// Makes item `indice` the selection. In a multi-select list `LB_SETCURSEL` is
/// ignored, so the previous selection is cleared and the item set explicitly.
fn marcar_indice(h: Hwnd, mensajes: &MensajesLista, indice: usize) {
    // SAFETY: Mensajes con enteros sobre el HWND recibido; sin punteros.
    unsafe {
        if mensajes.elegir == LB_SETCURSEL && es_multiseleccion(h) {
            SendMessageW(h, LB_SETSEL, 0, -1);
            SendMessageW(h, LB_SETSEL, 1, isize::try_from(indice).unwrap_or(0));
            return;
        }
        SendMessageW(h, mensajes.elegir, indice, 0);
    }
}
fn elegir_item(h: Hwnd, mensajes: &MensajesLista, prefijo: &str) -> bool {
    let texto: Vec<u16> = prefijo.encode_utf16().chain(std::iter::once(0)).collect();
    // SAFETY: `texto` vive durante la llamada síncrona (Windows la serializa para el
    // control de otro proceso); el resto son enteros sobre HWND recibidos.
    unsafe {
        let indice = SendMessageW(
            h,
            mensajes.buscar,
            DESDE_EL_PRINCIPIO,
            texto.as_ptr() as isize,
        );
        if indice == CB_ERR {
            return false;
        }
        marcar_indice(h, mensajes, usize::try_from(indice).unwrap_or(0));
        let id = usize::try_from(GetDlgCtrlID(h)).unwrap_or(0) & 0xFFFF;
        SendMessageW(GetParent(h), WM_COMMAND, (CBN_SELCHANGE << 16) | id, h);
    }
    true
}

/// Selects the first combo-box item starting with `prefijo` and notifies the
/// owning form as a user choice would. `false` when no item matches.
pub(crate) fn elegir_en_combo(h: Hwnd, prefijo: &str) -> bool {
    elegir_item(h, &COMBO, prefijo)
}

/// Same for a list box (`TListBox`): the first item starting with `prefijo`.
pub(crate) fn elegir_en_lista(h: Hwnd, prefijo: &str) -> bool {
    elegir_item(h, &LISTA, prefijo)
}

/// Picks an item by prefix in a combo box or a list box, by control class.
pub(crate) fn elegir_item_por_clase(h: Hwnd, clase: &str, prefijo: &str) -> bool {
    if clase == "TListBox" {
        elegir_en_lista(h, prefijo)
    } else {
        elegir_en_combo(h, prefijo)
    }
}

#[cfg(test)]
#[path = "../../../../tests/unit/adapters/gui/ffi/controles.rs"]
mod tests;
