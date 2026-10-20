//! Window actions: clicks, messages, physical inputs and desktop interactivity.

use super::ffi::{
    self, Hwnd, BM_CLICK, GA_ROOT, SW_RESTORE, WM_CLOSE, WM_LBUTTONDOWN, WM_LBUTTONUP,
};
use super::ventanas::{class_of, rect_of, resolve, ventanas_de, window_json};
use crate::args::req_str;
use crate::error::{Error, Result};
use serde_json::{json, Value};

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
        MecanismoClic::BotonEstandar => ffi::post_message(h, BM_CLICK, 0, 0),
        MecanismoClic::Raton => {
            let cr = ffi::rect_cliente(h);
            let x = (cr.right / 2).max(1) as isize;
            let y = (cr.bottom / 2).max(1) as isize;
            let lp = (x & 0xFFFF) | ((y & 0xFFFF) << 16);
            ffi::post_message(h, WM_LBUTTONDOWN, 1, lp) && ffi::post_message(h, WM_LBUTTONUP, 0, lp)
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
    ffi::send_message(h, msg, wparam, lparam);
    Ok(())
}

fn ejecutar_accion(h: Hwnd, action: &str, args: &Value) -> Result<()> {
    match action {
        "click" => clic_por_mensajes(h)?,
        "cerrar" => {
            ffi::post_message(h, WM_CLOSE, 0, 0);
        }
        "activar" => {
            ffi::mostrar_ventana(h, SW_RESTORE);
            ffi::traer_al_frente(h);
        }
        "mensaje" => enviar_mensaje(h, args)?,
        "clic_fisico" => {
            let r = rect_of(h);
            if r.right <= r.left {
                return Err(Error::Gui(format!("Control 0x{h:X} has no rectangle")));
            }
            ffi::clic_fisico_en((r.left + r.right) / 2, (r.top + r.bottom) / 2);
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
    ffi::clic_fisico_en(x, y);
}

pub(crate) fn escritorio_interactivo() -> bool {
    ffi::escritorio_disponible()
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
    ffi::traer_al_frente(ffi::get_ancestor(h, GA_ROOT));
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
