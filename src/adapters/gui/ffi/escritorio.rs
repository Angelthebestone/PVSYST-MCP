//! Private Win32 desktop: create it, attach a thread to it, start a process on
//! it, and post the mouse and keyboard input a user would type there. A
//! process on a non-input desktop receives no real input, so everything the
//! MCP "types" in that mode is posted to the window as messages.

use super::puntos::{clic_en_pantalla, ventana_en};
use super::ventanas::{post_message, primer_plano, GetWindowThreadProcessId};
use super::Hwnd;
use std::sync::atomic::{AtomicBool, Ordering};

/// `GENERIC_ALL`: the MCP owns the desktop it creates.
const ACCESO_TOTAL: u32 = 0x1000_0000;
const WM_KEYDOWN: u32 = 0x0100;
const WM_KEYUP: u32 = 0x0101;
const WM_SYSKEYDOWN: u32 = 0x0104;
const WM_SYSKEYUP: u32 = 0x0105;
const VK_MENU: u8 = 0x12;
/// `lParam` bit 29: Alt is down (makes a key a system key).
const LPARAM_ALT: isize = 1 << 29;
/// `lParam` bits 30-31 of a key-up: previous state down, transition to up.
const LPARAM_SOLTAR: isize = 0xC000_0000u32 as isize;
const REPETICION_UNICA: isize = 1;
/// Fields before `rcCaret` in `GUITHREADINFO`, used to size the struct.
const FLAGS_CREACION_DEFECTO: u32 = 0;

static AISLADO: AtomicBool = AtomicBool::new(false);

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

#[repr(C)]
struct InfoInicio {
    tamano: u32,
    reservado: *mut u16,
    escritorio: *mut u16,
    titulo: *mut u16,
    x: u32,
    y: u32,
    ancho: u32,
    alto: u32,
    columnas: u32,
    filas: u32,
    relleno: u32,
    banderas: u32,
    mostrar: u16,
    reservado2_tam: u16,
    reservado2: *mut u8,
    entrada: isize,
    salida: isize,
    error: isize,
}

#[repr(C)]
#[derive(Default)]
struct InfoProceso {
    proceso: isize,
    hilo: isize,
    pid: u32,
    id_hilo: u32,
}

#[link(name = "user32")]
extern "system" {
    fn CreateDesktopW(
        nombre: *const u16,
        dispositivo: *const u16,
        modo: *const u8,
        banderas: u32,
        acceso: u32,
        atributos: *const u8,
    ) -> isize;
    fn SetThreadDesktop(escritorio: isize) -> i32;
    fn GetGUIThreadInfo(hilo: u32, info: *mut InfoHiloGui) -> i32;
}

#[link(name = "kernel32")]
extern "system" {
    fn CreateProcessW(
        aplicacion: *const u16,
        linea: *mut u16,
        atributos_proceso: *const u8,
        atributos_hilo: *const u8,
        hereda: i32,
        banderas: u32,
        entorno: *const u8,
        directorio: *const u16,
        inicio: *const InfoInicio,
        proceso: *mut InfoProceso,
    ) -> i32;
    fn GetLastError() -> u32;
    fn CloseHandle(h: isize) -> i32;
}

/// Whether this process works on its own private desktop.
pub(crate) fn aislado() -> bool {
    AISLADO.load(Ordering::Relaxed)
}

/// Marks the process as working on a private desktop (or not).
pub(crate) fn fijar_aislado(valor: bool) {
    AISLADO.store(valor, Ordering::Relaxed);
}

fn utf16_nulo(texto: &str) -> Vec<u16> {
    texto.encode_utf16().chain(std::iter::once(0)).collect()
}

/// Creates the desktop `nombre`, or opens it when it already exists. Returns
/// its handle, or the Win32 error code.
pub(crate) fn crear_escritorio(nombre: &str) -> Result<isize, u32> {
    let nombre = utf16_nulo(nombre);
    // SAFETY: `nombre` es un buffer UTF-16 terminado en nulo vivo durante la
    // llamada; el resto de argumentos son punteros nulos permitidos por la API.
    let h = unsafe {
        CreateDesktopW(
            nombre.as_ptr(),
            std::ptr::null(),
            std::ptr::null(),
            FLAGS_CREACION_DEFECTO,
            ACCESO_TOTAL,
            std::ptr::null(),
        )
    };
    if h == 0 {
        // SAFETY: Lectura del último error del hilo, sin punteros.
        return Err(unsafe { GetLastError() });
    }
    Ok(h)
}

/// Attaches the calling thread to `escritorio`. It must have no windows or
/// hooks yet. Returns the Win32 error code on failure.
pub(crate) fn adjuntar_hilo(escritorio: isize) -> Result<(), u32> {
    // SAFETY: Cambia el escritorio del hilo actual con un handle propio vivo.
    if unsafe { SetThreadDesktop(escritorio) } != 0 {
        return Ok(());
    }
    // SAFETY: Lectura del último error del hilo, sin punteros.
    Err(unsafe { GetLastError() })
}

/// Starts `linea` (a full command line, `exe` first) on the desktop `nombre`.
/// Returns the process id, or the Win32 error code.
pub(crate) fn lanzar_en_escritorio(
    linea: &str,
    directorio: Option<&str>,
    nombre: &str,
) -> Result<u32, u32> {
    let mut linea = utf16_nulo(linea);
    let mut escritorio = utf16_nulo(&format!("winsta0\\{nombre}"));
    let directorio = directorio.map(utf16_nulo);
    let inicio = InfoInicio {
        tamano: std::mem::size_of::<InfoInicio>() as u32,
        reservado: std::ptr::null_mut(),
        escritorio: escritorio.as_mut_ptr(),
        titulo: std::ptr::null_mut(),
        x: 0,
        y: 0,
        ancho: 0,
        alto: 0,
        columnas: 0,
        filas: 0,
        relleno: 0,
        banderas: 0,
        mostrar: 0,
        reservado2_tam: 0,
        reservado2: std::ptr::null_mut(),
        entrada: 0,
        salida: 0,
        error: 0,
    };
    let mut proceso = InfoProceso::default();
    // SAFETY: `linea`, `escritorio` y `directorio` son buffers UTF-16 nulos vivos
    // durante la llamada; `inicio` y `proceso` son locales con el layout de la API.
    let ok = unsafe {
        CreateProcessW(
            std::ptr::null(),
            linea.as_mut_ptr(),
            std::ptr::null(),
            std::ptr::null(),
            0,
            0,
            std::ptr::null(),
            directorio.as_ref().map_or(std::ptr::null(), |d| d.as_ptr()),
            &inicio,
            &mut proceso,
        )
    };
    if ok == 0 {
        // SAFETY: Lectura del último error del hilo, sin punteros.
        return Err(unsafe { GetLastError() });
    }
    // SAFETY: Handles devueltos por `CreateProcessW`, cerrados una sola vez.
    unsafe {
        CloseHandle(proceso.proceso);
        CloseHandle(proceso.hilo);
    }
    Ok(proceso.pid)
}

/// Posts a left click at the screen point `(x, y)` to the window under it.
pub(crate) fn clic_emulado_en(x: i32, y: i32) {
    let h = ventana_en(x, y);
    if h != 0 {
        // A click that cannot be queued has nowhere to report to here: the
        // callers re-read the form, which shows it did not happen.
        let _ = clic_en_pantalla(h, (x, y));
    }
}

/// Window of the foreground application that has the keyboard focus.
fn ventana_con_foco() -> Hwnd {
    let primero = primer_plano();
    let mut hilo = 0u32;
    // SAFETY: `hilo` es una local viva; la API solo escribe un u32 en ella.
    let id = unsafe { GetWindowThreadProcessId(primero, &mut hilo) };
    let mut info = InfoHiloGui {
        tamano: std::mem::size_of::<InfoHiloGui>() as u32,
        ..InfoHiloGui::default()
    };
    // SAFETY: `info` es una local con el layout de GUITHREADINFO y su tamaño.
    let ok = unsafe { GetGUIThreadInfo(id, &mut info) };
    if ok != 0 && info.foco != 0 {
        return info.foco;
    }
    primero
}

/// Posts a key press (and release) to the focused window. `con_alt` makes it
/// a system key, which is what opens menus. Ctrl and Shift are not
/// representable in posted messages (the target reads them from the keyboard
/// state), so callers must refuse those combinations.
pub(crate) fn tecla_emulada(vk: u8, con_alt: bool) {
    let h = ventana_con_foco();
    let (abajo, arriba, alt) = if con_alt || vk == VK_MENU {
        (WM_SYSKEYDOWN, WM_SYSKEYUP, LPARAM_ALT)
    } else {
        (WM_KEYDOWN, WM_KEYUP, 0)
    };
    post_message(h, abajo, usize::from(vk), REPETICION_UNICA | alt);
    post_message(
        h,
        arriba,
        usize::from(vk),
        REPETICION_UNICA | alt | LPARAM_SOLTAR,
    );
}
