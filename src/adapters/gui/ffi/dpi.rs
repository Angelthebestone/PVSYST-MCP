//! DPI awareness, monitor scaling and client coordinates.

use super::puntos::Punto;
use super::Hwnd;

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

#[link(name = "user32")]
extern "system" {
    fn ClientToScreen(h: Hwnd, p: *mut Punto) -> i32;
    fn GetDpiForWindow(h: Hwnd) -> u32;
    fn GetWindowDpiAwarenessContext(h: Hwnd) -> isize;
    fn GetAwarenessFromDpiAwarenessContext(contexto: isize) -> i32;
    fn MonitorFromWindow(h: Hwnd, flags: u32) -> isize;
    fn SetProcessDpiAwarenessContext(contexto: isize) -> i32;
}

#[link(name = "shcore")]
extern "system" {
    fn GetDpiForMonitor(monitor: isize, tipo: u32, x: *mut u32, y: *mut u32) -> i32;
}

/// Screen position of the client-area origin of `h`.
pub(crate) fn origen_cliente(h: Hwnd) -> (i32, i32) {
    let mut p = Punto::default();
    // SAFETY: `p` es una estructura local en la pila con el layout exacto de POINT;
    // la API `ClientToScreen` únicamente escribe las coordenadas (x, y) en ella.
    unsafe {
        ClientToScreen(h, &mut p);
    }
    (p.x, p.y)
}

/// Whether `h` belongs to a DPI-unaware process: Windows virtualises its
/// coordinates and bitmaps to a 96-DPI layout.
pub(crate) fn ventana_ignora_dpi(h: Hwnd) -> bool {
    // SAFETY: Consultas Win32 de solo lectura pasando el HWND validado;
    // devuelven contextos enteros por valor sin desreferenciar punteros.
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
    // SAFETY: Fija el modo de DPI del proceso actual pasando una constante entera por valor;
    // no interactúa con punteros ni memoria del llamador.
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
    // SAFETY: `x` e `y` son enteros locales vivos en la pila donde la API escribe;
    // `MonitorFromWindow` devuelve un HMONITOR válido o 0 si no se encuentra.
    let de_monitor = unsafe {
        let monitor = MonitorFromWindow(h, MONITOR_DEFAULTTONEAREST);
        monitor != 0 && GetDpiForMonitor(monitor, MDT_EFFECTIVE_DPI, &mut x, &mut y) == 0
    };
    if de_monitor && x != 0 {
        return x;
    }
    // SAFETY: Consulta Win32 de solo lectura pasando el HWND validado por valor.
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

/// `v` laid out at 96 DPI, scaled to `dpi`. Complexity: O(1).
pub(crate) fn escalar_a_dpi(v: i32, dpi: u32) -> i32 {
    let base = i64::from(DPI_BASE);
    let escalado = i64::from(v) * i64::from(dpi) / base;
    i32::try_from(escalado).unwrap_or(v)
}

#[cfg(test)]
#[path = "../../../../tests/unit/adapters/gui/ffi/dpi.rs"]
mod tests;
