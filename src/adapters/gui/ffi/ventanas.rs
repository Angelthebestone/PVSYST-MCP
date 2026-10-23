//! Window enumeration, inspection and messaging.

use super::{EnumProc, Hwnd};

pub(crate) const WM_CLOSE: u32 = 0x0010;
pub(crate) const WM_LBUTTONDOWN: u32 = 0x0201;
pub(crate) const WM_LBUTTONUP: u32 = 0x0202;
pub(crate) const BM_CLICK: u32 = 0x00F5;
pub(crate) const SW_RESTORE: i32 = 9;
pub(crate) const GA_ROOT: u32 = 2;
pub(crate) const TH32CS_SNAPPROCESS: u32 = 0x0000_0002;
const SMTO_ABORTIFHUNG: u32 = 0x0002;
pub(crate) const TIMEOUT_MENSAJE_MS: u32 = 3000;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) struct Mensaje {
    pub(crate) id: u32,
    pub(crate) wparam: usize,
    pub(crate) lparam: isize,
}

impl Mensaje {
    pub(crate) const fn nuevo(id: u32, wparam: usize, lparam: isize) -> Self {
        Self { id, wparam, lparam }
    }

    pub(crate) const fn simple(id: u32) -> Self {
        Self {
            id,
            wparam: 0,
            lparam: 0,
        }
    }
}

#[repr(C)]
#[derive(Clone, Copy, Default)]
pub(crate) struct Rect {
    pub(crate) left: i32,
    pub(crate) top: i32,
    pub(crate) right: i32,
    pub(crate) bottom: i32,
}

#[repr(C)]
pub(crate) struct ProcessEntry32W {
    pub(crate) dw_size: u32,
    pub(crate) cnt_usage: u32,
    pub(crate) th32_process_id: u32,
    pub(crate) th32_default_heap_id: usize,
    pub(crate) th32_module_id: u32,
    pub(crate) cnt_threads: u32,
    pub(crate) th32_parent_process_id: u32,
    pub(crate) pc_pri_class_base: i32,
    pub(crate) dw_flags: u32,
    pub(crate) sz_exe_file: [u16; 260],
}

#[link(name = "user32")]
extern "system" {
    fn EnumWindows(cb: EnumProc, l: isize) -> i32;
    fn EnumChildWindows(parent: Hwnd, cb: EnumProc, l: isize) -> i32;
    pub(crate) fn GetWindowTextW(h: Hwnd, s: *mut u16, n: i32) -> i32;
    pub(crate) fn GetClassNameW(h: Hwnd, s: *mut u16, n: i32) -> i32;
    pub(crate) fn GetWindowThreadProcessId(h: Hwnd, pid: *mut u32) -> u32;
    fn IsWindowVisible(h: Hwnd) -> i32;
    fn IsWindowEnabled(h: Hwnd) -> i32;
    fn IsWindow(h: Hwnd) -> i32;
    fn IsIconic(h: Hwnd) -> i32;
    pub(crate) fn GetWindowRect(h: Hwnd, r: *mut Rect) -> i32;
    fn GetClientRect(h: Hwnd, r: *mut Rect) -> i32;
    fn SendMessageTimeoutW(
        h: Hwnd,
        msg: u32,
        w: usize,
        l: isize,
        flags: u32,
        timeout: u32,
        res: *mut isize,
    ) -> isize;
    fn PostMessageW(h: Hwnd, msg: u32, w: usize, l: isize) -> i32;
    fn ShowWindow(h: Hwnd, cmd: i32) -> i32;
    fn SetForegroundWindow(h: Hwnd) -> i32;
    fn GetAncestor(h: Hwnd, flags: u32) -> Hwnd;
    fn GetForegroundWindow() -> Hwnd;
}

#[link(name = "kernel32")]
extern "system" {
    pub(crate) fn CreateToolhelp32Snapshot(flags: u32, pid: u32) -> isize;
    pub(crate) fn Process32FirstW(snap: isize, pe: *mut ProcessEntry32W) -> i32;
    pub(crate) fn Process32NextW(snap: isize, pe: *mut ProcessEntry32W) -> i32;
    pub(crate) fn CloseHandle(h: isize) -> i32;
}

unsafe extern "system" fn collect_enum(h: Hwnd, l: isize) -> i32 {
    let out = &mut *(l as *mut Vec<Hwnd>);
    out.push(h);
    1
}

pub(crate) fn enum_windows() -> Vec<Hwnd> {
    let mut v: Vec<Hwnd> = Vec::new();
    // SAFETY: `collect_enum` solo reinterpreta `lparam` como el `Vec<Hwnd>` local `v`, vivo y sin alias durante la enumeración síncrona.
    unsafe {
        EnumWindows(Some(collect_enum), &mut v as *mut _ as isize);
    }
    v
}

pub(crate) fn enum_children(parent: Hwnd) -> Vec<Hwnd> {
    let mut v: Vec<Hwnd> = Vec::new();
    if !es_ventana(parent) {
        return v;
    }
    // SAFETY: `collect_enum` solo reinterpreta `lparam` como el `Vec<Hwnd>` local `v`, vivo y sin alias durante la enumeración síncrona.
    unsafe {
        EnumChildWindows(parent, Some(collect_enum), &mut v as *mut _ as isize);
    }
    v
}

/// Cheap window state flags used by the JSON snapshots.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) struct EstadoVentana {
    pub(crate) visible: bool,
    pub(crate) habilitada: bool,
    pub(crate) minimizada: bool,
}

pub(crate) fn estado_ventana(h: Hwnd) -> EstadoVentana {
    if !es_ventana(h) {
        return EstadoVentana::default();
    }
    // SAFETY: Consultas de solo lectura sobre un HWND validado; devuelven enteros por valor.
    unsafe {
        EstadoVentana {
            visible: IsWindowVisible(h) != 0,
            habilitada: IsWindowEnabled(h) != 0,
            minimizada: IsIconic(h) != 0,
        }
    }
}

pub(crate) fn es_ventana(h: Hwnd) -> bool {
    if h == 0 {
        return false;
    }
    // SAFETY: Consulta de solo lectura sobre un HWND; un handle inválido devuelve 0 de forma segura.
    unsafe { IsWindow(h) != 0 }
}

pub(crate) fn post_message(h: Hwnd, msg: Mensaje) -> bool {
    if !es_ventana(h) {
        return false;
    }
    // SAFETY: `PostMessageW` encola enteros sobre un HWND validado sin retener punteros.
    unsafe { PostMessageW(h, msg.id, msg.wparam, msg.lparam) != 0 }
}

pub(crate) fn send_message_timeout(h: Hwnd, msg: Mensaje, timeout_ms: u32) -> Option<isize> {
    if !es_ventana(h) {
        return None;
    }
    let mut respuesta = 0isize;
    // SAFETY: `SendMessageTimeoutW` previene bloqueos infinitos usando SMTO_ABORTIFHUNG;
    // `h` está validado como ventana activa. Si `msg.lparam` contiene un puntero (ej. búfer
    // para WM_GETTEXT o CB_GETLBTEXT), el llamador garantiza que el búfer está vivo durante
    // la llamada síncrona y tiene la longitud indicada en `msg.wparam`.
    // `respuesta` es una variable local viva de 64 bits donde la API escribe el resultado.
    let ok = unsafe {
        SendMessageTimeoutW(
            h,
            msg.id,
            msg.wparam,
            msg.lparam,
            SMTO_ABORTIFHUNG,
            timeout_ms,
            &mut respuesta,
        )
    };
    (ok != 0).then_some(respuesta)
}

pub(crate) fn send_message_res(h: Hwnd, msg: Mensaje) -> isize {
    send_message_timeout(h, msg, TIMEOUT_MENSAJE_MS).unwrap_or(0)
}

pub(crate) fn send_message(h: Hwnd, msg: Mensaje) {
    let _ = send_message_res(h, msg);
}

pub(crate) fn mostrar_ventana(h: Hwnd, cmd: i32) {
    if !es_ventana(h) {
        return;
    }
    // SAFETY: `ShowWindow` solo actúa sobre el HWND validado; cmd se pasa por valor.
    unsafe {
        ShowWindow(h, cmd);
    }
}

pub(crate) fn traer_al_frente(h: Hwnd) {
    if !es_ventana(h) {
        return;
    }
    // SAFETY: `SetForegroundWindow` solo actúa sobre el HWND validado.
    unsafe {
        SetForegroundWindow(h);
    }
}

pub(crate) fn get_ancestor(h: Hwnd, flags: u32) -> Hwnd {
    if !es_ventana(h) {
        return 0;
    }
    // SAFETY: Consulta de solo lectura sobre un HWND validado.
    unsafe { GetAncestor(h, flags) }
}

pub(crate) fn rect_cliente(h: Hwnd) -> Rect {
    let mut r = Rect::default();
    if !es_ventana(h) {
        return r;
    }
    // SAFETY: `r` es una local viva con el layout exacto de RECT; la API solo escribe en ella.
    unsafe {
        GetClientRect(h, &mut r);
    }
    r
}

pub(crate) fn primer_plano() -> Hwnd {
    // SAFETY: Sin argumentos; devuelve 0 si no hay ventana en primer plano.
    unsafe { GetForegroundWindow() }
}
