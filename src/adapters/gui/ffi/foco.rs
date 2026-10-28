//! Keyboard focus of another process's controls: read it and move it.

use super::ventanas::GetWindowThreadProcessId;
use super::Hwnd;

#[repr(C)]
#[derive(Default)]
struct InfoHiloGui {
    tamano: u32,
    banderas: u32,
    activa: isize,
    foco: isize,
    captura: isize,
    duena_del_menu: isize,
    mover: isize,
    cursor_texto: isize,
    rc_cursor: [i32; 4],
}

#[link(name = "user32")]
extern "system" {
    fn GetGUIThreadInfo(hilo: u32, info: *mut InfoHiloGui) -> i32;
    fn AttachThreadInput(origen: u32, destino: u32, unir: i32) -> i32;
    fn SetFocus(h: Hwnd) -> Hwnd;
}

#[link(name = "kernel32")]
extern "system" {
    fn GetCurrentThreadId() -> u32;
}

const UNIR: i32 = 1;
const SEPARAR: i32 = 0;

/// Control that has the keyboard focus in thread `hilo`, or 0 when none.
pub(crate) fn foco_del_hilo(hilo: u32) -> Hwnd {
    let mut info = InfoHiloGui {
        tamano: std::mem::size_of::<InfoHiloGui>() as u32,
        ..InfoHiloGui::default()
    };
    // SAFETY: `info` es una local con el layout de GUITHREADINFO y su tamaño.
    let ok = unsafe { GetGUIThreadInfo(hilo, &mut info) };
    if ok == 0 {
        return 0;
    }
    info.foco
}

/// Thread that owns the window `h`.
pub(crate) fn hilo_de(h: Hwnd) -> u32 {
    let mut pid = 0u32;
    // SAFETY: `pid` es una local viva; la API solo escribe un u32 en ella.
    unsafe { GetWindowThreadProcessId(h, &mut pid) }
}

/// Gives the keyboard focus to the control `h` of another process and reports
/// whether it really has it afterwards. Focus lives per thread input queue, so
/// the call shares the target's queue for the instant `SetFocus` needs.
pub(crate) fn dar_foco(h: Hwnd) -> bool {
    let hilo = hilo_de(h);
    // SAFETY: Sin punteros; devuelve el id del hilo llamante.
    let propio = unsafe { GetCurrentThreadId() };
    // SAFETY: Une y separa las colas de entrada de dos hilos por id; `h` ya es una ventana validada por el llamador.
    unsafe {
        AttachThreadInput(propio, hilo, UNIR);
        SetFocus(h);
        AttachThreadInput(propio, hilo, SEPARAR);
    }
    foco_del_hilo(hilo) == h
}
