//! Window actions: clicks, messages, physical inputs and desktop interactivity.

use super::ffi::{
    self, Hwnd, Rect, BM_CLICK, DESKTOP_READOBJECTS, GA_ROOT, MOUSEEVENTF_LEFTDOWN,
    MOUSEEVENTF_LEFTUP, SW_RESTORE, WM_CLOSE, WM_LBUTTONDOWN, WM_LBUTTONUP,
};
use super::ventanas::{class_of, rect_of, resolve, ventanas_de, window_json};
use crate::args::req_str;
use crate::error::{Error, Result};
use serde_json::{json, Value};

const CLICK_SETTLE_MS: u64 = 150;

/// Window classes (Win32 and VCL) that implement `BM_CLICK`.
const CLASES_BOTON: &[&str] = &["Button", "TButton", "TBitBtn", "TCheckBox", "TRadioButton"];

/// How a control is clicked through messages. Exactly one mechanism is used:
/// a button handles both `BM_CLICK` and mouse messages, so sending both would
/// activate it twice.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum MecanismoClic {
    BotonEstandar,
    Raton,
}

pub(crate) fn mecanismo_para(clase: &str) -> MecanismoClic {
    if CLASES_BOTON.iter().any(|c| c.eq_ignore_ascii_case(clase)) {
        MecanismoClic::BotonEstandar
    } else {
        MecanismoClic::Raton
    }
}

pub(crate) fn clic_por_mensajes(h: Hwnd) -> Result<()> {
    let encolado = match mecanismo_para(&class_of(h)) {
        // SAFETY: `PostMessageW` encola mensajes sin punteros; un HWND inválido solo hace fallar la llamada.
        MecanismoClic::BotonEstandar => unsafe { ffi::PostMessageW(h, BM_CLICK, 0, 0) != 0 },
        MecanismoClic::Raton => {
            let mut cr = Rect::default();
            // SAFETY: `cr` es una local viva y exclusiva durante la llamada; `GetClientRect` solo escribe en ella.
            unsafe {
                ffi::GetClientRect(h, &mut cr);
            }
            let x = (cr.right / 2).max(1) as isize;
            let y = (cr.bottom / 2).max(1) as isize;
            let lp = (x & 0xFFFF) | ((y & 0xFFFF) << 16);
            // SAFETY: `PostMessageW` encola mensajes sin punteros; un HWND inválido solo hace fallar la llamada.
            unsafe {
                ffi::PostMessageW(h, WM_LBUTTONDOWN, 1, lp) != 0
                    && ffi::PostMessageW(h, WM_LBUTTONUP, 0, lp) != 0
            }
        }
    };
    if encolado {
        Ok(())
    } else {
        Err(Error::Gui(format!(
            "PostMessage falló para el control 0x{h:X}"
        )))
    }
}

fn enviar_mensaje(h: Hwnd, args: &Value) -> Result<()> {
    let msg = args
        .get("mensaje")
        .and_then(serde_json::Value::as_u64)
        .ok_or_else(|| Error::Invalid("accion 'mensaje' requires numeric 'mensaje'".into()))?
        as u32;
    let wparam = args
        .get("wparam")
        .and_then(serde_json::Value::as_u64)
        .unwrap_or(0) as usize;
    let lparam = args
        .get("lparam")
        .and_then(serde_json::Value::as_i64)
        .unwrap_or(0) as isize;
    // SAFETY: `wparam`/`lparam` son enteros que el llamador decide pasar al control; no se desreferencian aquí.
    unsafe {
        ffi::SendMessageW(h, msg, wparam, lparam);
    }
    Ok(())
}

fn ejecutar_accion(h: Hwnd, action: &str, args: &Value) -> Result<()> {
    match action {
        "click" => clic_por_mensajes(h)?,
        // SAFETY: `PostMessageW` encola WM_CLOSE sin punteros; un HWND inválido solo hace fallar la llamada.
        "cerrar" => unsafe {
            ffi::PostMessageW(h, WM_CLOSE, 0, 0);
        },
        // SAFETY: `ShowWindow` y `SetForegroundWindow` solo reciben el HWND `h`; con un handle inválido fallan sin efecto.
        "activar" => unsafe {
            ffi::ShowWindow(h, SW_RESTORE);
            ffi::SetForegroundWindow(h);
        },
        "mensaje" => enviar_mensaje(h, args)?,
        "clic_fisico" => {
            let r = rect_of(h);
            if r.right <= r.left {
                return Err(Error::Gui(format!("Control 0x{h:X} has no rectangle")));
            }
            clic_fisico_en((r.left + r.right) / 2, (r.top + r.bottom) / 2);
        }
        other => {
            return Err(Error::Invalid(format!(
                "Unknown accion '{other}' (click|clic_fisico|cerrar|activar|mensaje)"
            )))
        }
    }
    Ok(())
}

pub(crate) fn clic_fisico_en(x: i32, y: i32) {
    // SAFETY: Funciones de entrada sin punteros: mueven el cursor y emiten eventos de ratón.
    unsafe {
        ffi::SetCursorPos(x, y);
        std::thread::sleep(std::time::Duration::from_millis(CLICK_SETTLE_MS));
        ffi::mouse_event(MOUSEEVENTF_LEFTDOWN, 0, 0, 0, 0);
        ffi::mouse_event(MOUSEEVENTF_LEFTUP, 0, 0, 0, 0);
    }
}

pub(crate) fn escritorio_interactivo() -> bool {
    // SAFETY: Sin punteros del llamador; el handle devuelto se cierra con `CloseDesktop`.
    let escritorio = unsafe { ffi::OpenInputDesktop(0, 0, DESKTOP_READOBJECTS) };
    if escritorio == 0 {
        return false;
    }
    // SAFETY: `escritorio` es el handle no nulo recién abierto y se cierra una sola vez.
    unsafe { ffi::CloseDesktop(escritorio) };
    // SAFETY: Sin argumentos; devuelve 0 si no hay ventana en primer plano.
    let foreground = unsafe { ffi::GetForegroundWindow() };
    foreground != 0
}

pub(crate) fn exigir_escritorio() -> Result<()> {
    if escritorio_interactivo() {
        Ok(())
    } else {
        Err(Error::Gui("Escritorio no interactivo".into()))
    }
}

pub(crate) fn invocar(_paths: &crate::config::Paths, args: &Value) -> Result<Value> {
    let scoped = ventanas_de(args)?;
    let reference = req_str(
        args,
        &["ventana"],
        "hwnd, class or title of the control to invoke",
    )?;
    let h = resolve(&scoped, reference)
        .ok_or_else(|| Error::Gui(format!("Control '{reference}' not found")))?;

    let action = args
        .get("accion")
        .and_then(|v| v.as_str())
        .unwrap_or("click");
    // SAFETY: `GetAncestor` y `SetForegroundWindow` solo leen/actúan sobre el HWND `h`.
    unsafe {
        ffi::SetForegroundWindow(ffi::GetAncestor(h, GA_ROOT));
    }
    ejecutar_accion(h, action, args)?;

    Ok(json!({
        "invoked": reference,
        "hwnd": format!("0x{:X}", h),
        "action": action,
        "window": window_json(h),
    }))
}

#[cfg(test)]
#[path = "../../../tests/unit/adapters/gui/acciones.rs"]
mod tests;
