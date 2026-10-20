//! Keyboard, mouse, desktop and message-loop entry points.

use super::ventanas::primer_plano;
use super::{Hwnd, WinEventProc};

pub(crate) const WM_KEYDOWN: u32 = 0x0100;
pub(crate) const WM_KEYUP: u32 = 0x0101;
pub(crate) const WM_TIMER: u32 = 0x0113;
pub(crate) const EVENT_OBJECT_SHOW: u32 = 0x8002;
pub(crate) const WINEVENT_OUTOFCONTEXT: u32 = 0x0000;
pub(crate) const OBJID_WINDOW: i32 = 0;
pub(crate) const DESKTOP_READOBJECTS: u32 = 0x0001;
pub(crate) const MOUSEEVENTF_LEFTDOWN: u32 = 0x0002;
pub(crate) const MOUSEEVENTF_LEFTUP: u32 = 0x0004;
pub(crate) const TEMPORIZADOR: usize = 1;
const CLICK_SETTLE_MS: u64 = 150;

#[repr(C)]
#[derive(Default)]
pub(crate) struct Msg {
    pub(crate) hwnd: Hwnd,
    pub(crate) message: u32,
    pub(crate) wparam: usize,
    pub(crate) lparam: isize,
    pub(crate) time: u32,
    pub(crate) pt_x: i32,
    pub(crate) pt_y: i32,
}

#[link(name = "user32")]
extern "system" {
    fn SetCursorPos(x: i32, y: i32) -> i32;
    fn mouse_event(flags: u32, dx: u32, dy: u32, data: u32, extra: usize);
    fn OpenInputDesktop(flags: u32, inherit: i32, access: u32) -> isize;
    fn CloseDesktop(h: isize) -> i32;
    fn SetWinEventHook(
        min: u32,
        max: u32,
        module: isize,
        cb: WinEventProc,
        pid: u32,
        tid: u32,
        flags: u32,
    ) -> isize;
    fn UnhookWinEvent(h: isize) -> i32;
    fn GetMessageW(msg: *mut Msg, h: Hwnd, min: u32, max: u32) -> i32;
    fn TranslateMessage(msg: *const Msg) -> i32;
    fn DispatchMessageW(msg: *const Msg) -> isize;
    fn SetTimer(h: Hwnd, id: usize, ms: u32, cb: isize) -> usize;
    fn KillTimer(h: Hwnd, id: usize) -> i32;
    fn PostQuitMessage(code: i32);
}

pub(crate) fn clic_fisico_en(x: i32, y: i32) {
    // SAFETY: Funciones de entrada sin punteros: mueven el cursor y emiten eventos de ratón.
    unsafe {
        SetCursorPos(x, y);
        std::thread::sleep(std::time::Duration::from_millis(CLICK_SETTLE_MS));
        mouse_event(MOUSEEVENTF_LEFTDOWN, 0, 0, 0, 0);
        mouse_event(MOUSEEVENTF_LEFTUP, 0, 0, 0, 0);
    }
}

pub(crate) fn escritorio_disponible() -> bool {
    // SAFETY: El handle devuelto se cierra aquí mismo; las demás llamadas no usan punteros.
    unsafe {
        let escritorio = OpenInputDesktop(0, 0, DESKTOP_READOBJECTS);
        if escritorio == 0 {
            return false;
        }
        CloseDesktop(escritorio);
        primer_plano() != 0
    }
}

pub(crate) fn salir_bucle() {
    // SAFETY: Envía la señal de finalización al bucle de mensajes de este hilo.
    unsafe {
        PostQuitMessage(0);
    }
}

pub(crate) fn bombear_mensajes() {
    loop {
        let mut m = Msg::default();
        // SAFETY: `m` tiene el layout de MSG y fue rellenado por `GetMessageW` en esta iteración.
        unsafe {
            let r = GetMessageW(&mut m, 0, 0, 0);
            if r <= 0 || m.message == WM_TIMER {
                break;
            }
            TranslateMessage(&m);
            DispatchMessageW(&m);
        }
    }
}

pub(crate) fn instalar_gancho_mostrar(cb: WinEventProc) -> isize {
    // SAFETY: `cb` tiene la firma WINEVENTPROC y se invoca en este hilo.
    unsafe {
        SetWinEventHook(
            EVENT_OBJECT_SHOW,
            EVENT_OBJECT_SHOW,
            0,
            cb,
            0,
            0,
            WINEVENT_OUTOFCONTEXT,
        )
    }
}

pub(crate) fn iniciar_temporizador(ms: u32) {
    // SAFETY: Sin ventana ni callback; el temporizador entrega WM_TIMER a esta cola.
    unsafe {
        SetTimer(0, TEMPORIZADOR, ms, 0);
    }
}

pub(crate) fn retirar_gancho(gancho: isize) {
    // SAFETY: `gancho` proviene de `instalar_gancho_mostrar` y se retira una sola vez.
    unsafe {
        KillTimer(0, TEMPORIZADOR);
        UnhookWinEvent(gancho);
    }
}
