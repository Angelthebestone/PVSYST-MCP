//! Window actions: clicks, messages, physical inputs and desktop interactivity.

use super::ffi::{self, Hwnd, BM_CLICK, GA_ROOT, SW_RESTORE, WM_CLOSE};
use super::rotulos::{rotulos_acciones, Idioma, RotulosAcciones};
use super::ventanas::{class_of, rect_of, resolve, text_of, ventanas_de, visible, window_json};
use crate::args::{opt_i32, opt_u32, req_str};
use crate::error::{Error, Result};
use serde_json::{json, Value};
use std::time::Duration;

/// Window classes (Win32 and VCL) that implement `BM_CLICK`.
const CLASES_BOTON: &[&str] = &["Button", "TButton", "TBitBtn", "TCheckBox", "TRadioButton"];
const ROTULOS: RotulosAcciones = rotulos_acciones(Idioma::Es);
/// Class of PVsyst's message boxes.
pub(crate) const CLASE_MENSAJE: &str = "TMessageForm";
/// Class of the madExcept crash dialog.
pub(crate) const CLASE_EXCEPCION: &str = "madExceptWndClass";
/// Lets the form process the close before the next tool call.
const PAUSA_CIERRE: Duration = Duration::from_millis(300);

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
        MecanismoClic::BotonEstandar => ffi::post_message(h, ffi::Mensaje::simple(BM_CLICK)),
        MecanismoClic::Raton => {
            let cr = ffi::rect_cliente(h);
            if cr.right <= 0 || cr.bottom <= 0 {
                return Err(Error::Gui(format!(
                    "El control 0x{h:X} no tiene área visible"
                )));
            }
            ffi::clic_en_cliente(h, (cr.right / 2, cr.bottom / 2))
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
    let msg = opt_u32(args, "mensaje")?
        .ok_or_else(|| Error::Invalid("accion 'mensaje' requires numeric 'mensaje'".into()))?;
    let wparam = opt_u32(args, "wparam")?.unwrap_or(0) as usize;
    let lparam = args.get("lparam").map_or(Ok(0), |v| {
        v.as_i64()
            .and_then(|n| isize::try_from(n).ok())
            .ok_or_else(|| Error::Invalid("'lparam' debe ser un entero".into()))
    })?;
    ffi::send_message_timeout(
        h,
        ffi::Mensaje::nuevo(msg, wparam, lparam),
        ffi::TIMEOUT_MENSAJE_MS,
    )
    .ok_or_else(|| Error::Gui("PVsyst no respondió en 3 s al enviar mensaje".into()))?;
    Ok(())
}

fn botones_de_cierre(h: Hwnd) -> &'static [&'static str] {
    match class_of(h).as_str() {
        CLASE_MENSAJE => ROTULOS.botones_mensaje,
        CLASE_EXCEPCION => ROTULOS.botones_excepcion,
        _ => ROTULOS.botones_cierre,
    }
}

/// Closes a VCL form the way a user does: its own close/cancel button first,
/// `WM_CLOSE` only when the form has none. A message box with no harmless
/// button (a Sí/No question) is left open and reported.
fn cerrar_ventana(h: Hwnd) -> Result<()> {
    let botones = botones_de_cierre(h);
    let boton = ffi::enum_children(h)
        .into_iter()
        .filter(|c| visible(*c))
        .find(|c| botones.contains(&text_of(*c).as_str()));
    match boton {
        Some(b) => clic_por_mensajes(b)?,
        None if class_of(h) == CLASE_MENSAJE => {
            return Err(Error::Gui(format!(
                "El mensaje 0x{h:X} es una pregunta: léela con gui_observar que:formulario o controles y responde con click sobre el botón elegido"
            )))
        }
        None => {
            ffi::post_message(h, ffi::Mensaje::simple(WM_CLOSE));
        }
    }
    std::thread::sleep(PAUSA_CIERRE);
    Ok(())
}

/// `dx`/`dy` of the request: a point of the control in px at 96 DPI from its
/// top-left corner. Both or neither: half a point has no meaning.
fn desplazamiento(args: &Value) -> Result<Option<(i32, i32)>> {
    match (opt_i32(args, "dx")?, opt_i32(args, "dy")?) {
        (Some(dx), Some(dy)) => Ok(Some((dx, dy))),
        (None, None) => Ok(None),
        _ => Err(Error::Invalid(
            "'dx' y 'dy' van juntos (enteros, px a 96 DPI desde la esquina del control)".into(),
        )),
    }
}

fn clic_fisico_en_control(h: Hwnd, desplazamiento: Option<(i32, i32)>) -> Result<()> {
    let r = rect_of(h);
    if r.right <= r.left {
        return Err(Error::Gui(format!("Control 0x{h:X} has no rectangle")));
    }
    let (x, y) = match desplazamiento {
        Some((dx, dy)) => {
            let dpi = ffi::dpi_de(h);
            (
                r.left + ffi::escalar_a_dpi(dx, dpi),
                r.top + ffi::escalar_a_dpi(dy, dpi),
            )
        }
        None => ((r.left + r.right) / 2, (r.top + r.bottom) / 2),
    };
    ffi::clic_fisico_en(x, y);
    Ok(())
}

fn ejecutar_accion(h: Hwnd, action: &str, args: &Value) -> Result<()> {
    match action {
        "click" => match desplazamiento(args)? {
            Some(punto) if ffi::clic_en_cliente(h, punto) => {}
            Some(_) => return Err(Error::Gui(format!("PostMessage falló para 0x{h:X}"))),
            None => clic_por_mensajes(h)?,
        },
        "cerrar" => cerrar_ventana(h)?,
        "activar" => {
            ffi::mostrar_ventana(h, SW_RESTORE);
            ffi::traer_al_frente(h);
        }
        "mensaje" => enviar_mensaje(h, args)?,
        "clic_fisico" => clic_fisico_en_control(h, desplazamiento(args)?)?,
        other => {
            return Err(Error::Invalid(format!(
                "Unknown accion '{other}' (click|clic_fisico|cerrar|activar|mensaje)"
            )))
        }
    }
    Ok(())
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

    let mut window = window_json(h);
    if action == "cerrar" {
        if let Some(map) = window.as_object_mut() {
            map.insert("closed".into(), json!(!visible(h)));
        }
    }
    Ok(json!({
        "invoked": reference,
        "hwnd": format!("0x{:X}", h),
        "action": action,
        "window": window,
    }))
}

#[cfg(test)]
#[path = "../../../tests/unit/adapters/gui/acciones.rs"]
mod tests;
