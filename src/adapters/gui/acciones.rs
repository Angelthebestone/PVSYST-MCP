//! Window actions: clicks, messages, physical inputs and desktop interactivity.

use super::conduccion::{esperar_cierre, formularios};
use super::ffi::{self, Hwnd, BM_CLICK, GA_ROOT, SW_RESTORE, WM_CLOSE};
use super::modal;
use super::rotulos::{rotulos_acciones, Idioma, RotulosAcciones};
use super::ventanas::{
    class_of, pid_of, rect_of, resolve, text_of, ventanas_de, visible, window_json,
};
use crate::args::{opt_i32, opt_u32, req_str};
use crate::error::{Error, Result};
use serde_json::{json, Value};
use std::time::Duration;

/// Window classes (Win32 and VCL) that implement `BM_CLICK`.
const CLASES_BOTON: &[&str] = &["Button", "TButton", "TBitBtn", "TCheckBox", "TRadioButton"];
/// Buttons that abandon a form: tried first when the request says `cancelar:true`.
const BOTONES_CANCELAR: &[&str] = &["Cancelar", "No"];
const ROTULOS: RotulosAcciones = rotulos_acciones(Idioma::Es);
/// Class of PVsyst's message boxes.
pub(crate) const CLASE_MENSAJE: &str = "TMessageForm";
/// Class of the madExcept crash dialog.
pub(crate) const CLASE_EXCEPCION: &str = "madExceptWndClass";
/// Longest wait for a form to disappear after asking it to close.
const ESPERA_CIERRE: Duration = Duration::from_secs(2);
/// Time PVsyst takes to show the parent form again after an editor closes.
const ESPERA_REAPARICION: Duration = Duration::from_millis(400);
/// The home screen is always behind: it is not a form the close reopened.
const INICIO: &str = "THomeForm";

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
                return Err(Error::Gui(format!("Control 0x{h:X} has no visible area")));
            }
            ffi::clic_en_cliente(h, (cr.right / 2, cr.bottom / 2))
        }
    };
    if encolado {
        Ok(())
    } else {
        Err(Error::Gui(format!(
            "PostMessage failed for control 0x{h:X}"
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
            .ok_or_else(|| Error::Invalid("'lparam' must be an integer".into()))
    })?;
    ffi::send_message_timeout(
        h,
        ffi::Mensaje::nuevo(msg, wparam, lparam),
        ffi::TIMEOUT_MENSAJE_MS,
    )
    .ok_or_else(|| Error::Gui("PVsyst did not answer in 3 s to the message".into()))?;
    Ok(())
}

fn botones_de_cierre(h: Hwnd) -> &'static [&'static str] {
    match class_of(h).as_str() {
        CLASE_MENSAJE => ROTULOS.botones_mensaje,
        CLASE_EXCEPCION => ROTULOS.botones_excepcion,
        _ => ROTULOS.botones_cierre,
    }
}

/// Buttons that close `h`: with `cancelar`, Cancelar/No come first.
fn botones_para_cerrar(h: Hwnd, cancelar: bool) -> Vec<&'static str> {
    let base = botones_de_cierre(h);
    if cancelar {
        BOTONES_CANCELAR.iter().chain(base).copied().collect()
    } else {
        base.to_vec()
    }
}

/// Closes a VCL form the way a user does: its own close/cancel button first,
/// `WM_CLOSE` only when the form has none. `cancelar:true` tries Cancelar/No
/// before the default buttons. A message box with no harmless button (a Sí/No
/// question) is left open and reported.
pub(crate) fn cerrar_ventana(h: Hwnd, args: &Value) -> Result<()> {
    let botones = botones_para_cerrar(h, crate::args::flag(args, &["cancelar"]));
    let boton = ffi::enum_children(h)
        .into_iter()
        .filter(|c| visible(*c))
        .find(|c| botones.contains(&text_of(*c).as_str()));
    match boton {
        Some(b) => clic_por_mensajes(b)?,
        None if class_of(h) == CLASE_MENSAJE => {
            return Err(Error::Gui(format!(
                "Message 0x{h:X} is a question: read it with gui_observar que:formulario or controles and answer with click on the chosen button"
            )))
        }
        None => {
            ffi::post_message(h, ffi::Mensaje::simple(WM_CLOSE));
        }
    }
    Ok(())
}

/// `dx`/`dy` of the request: a point of the control in px at 96 DPI from its
/// top-left corner. Both or neither: half a point has no meaning.
fn desplazamiento(args: &Value) -> Result<Option<(i32, i32)>> {
    match (opt_i32(args, "dx")?, opt_i32(args, "dy")?) {
        (Some(dx), Some(dy)) => Ok(Some((dx, dy))),
        (None, None) => Ok(None),
        _ => Err(Error::Invalid(
            "'dx' and 'dy' go together (integers, px at 96 DPI from the control corner)".into(),
        )),
    }
}

pub(crate) fn clic_fisico_en_control(h: Hwnd, desplazamiento: Option<(i32, i32)>) -> Result<()> {
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
            Some(_) => return Err(Error::Gui(format!("PostMessage failed for 0x{h:X}"))),
            None => clic_por_mensajes(h)?,
        },
        "cerrar" => cerrar_ventana(h, args)?,
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
        Err(Error::Gui("Desktop is not interactive".into()))
    }
}

/// A visible top-level window of the process, as seen before and after a close.
struct VentanaViva {
    h: Hwnd,
    clase: String,
    titulo: String,
}

fn ventanas_vivas(pid: u32) -> Vec<VentanaViva> {
    formularios(pid)
        .into_iter()
        .map(|v| VentanaViva {
            h: v,
            clase: class_of(v),
            titulo: text_of(v),
        })
        .collect()
}

/// Forms the close made visible again: an editor can reopen its parent selector
/// as a modal, and a bare `closed:true` would hide it from the caller.
fn formas_reabiertas(antes: &[VentanaViva], despues: &[VentanaViva], cerrada: Hwnd) -> Vec<Value> {
    despues
        .iter()
        .filter(|v| v.h != cerrada && v.clase != INICIO && !antes.iter().any(|a| a.h == v.h))
        .map(|v| {
            json!({
                "hwnd": format!("0x{:X}", v.h),
                "class": v.clase,
                "title": v.titulo,
            })
        })
        .collect()
}

/// Names the reopened form and the `cerrar` that closes it, so the caller does
/// not have to discover the class with another observation.
fn aviso_de_reaparicion(reabiertas: &[Value]) -> Option<String> {
    let primera = reabiertas.first()?;
    let clase = primera["class"].as_str().unwrap_or_default();
    let hwnd = primera["hwnd"].as_str().unwrap_or_default();
    let resto = reabiertas.len().saturating_sub(1);
    let extra = if resto == 0 {
        String::new()
    } else {
        format!(" (and {resto} more)")
    };
    Some(format!(
        "Closing reopened {clase}{extra}: close it too with gui_actuar accion:cerrar ventana:{hwnd}"
    ))
}

pub(crate) fn invocar(args: &Value) -> Result<Value> {
    let scoped = ventanas_de(args)?;
    let reference = req_str(
        args,
        &["ventana"],
        "hwnd, class or title of the control to invoke",
    )?;
    let action = args
        .get("accion")
        .and_then(|v| v.as_str())
        .unwrap_or("click");
    let h = resolve(&scoped, reference).ok_or_else(|| no_encontrado(reference, action))?;
    if !matches!(action, "activar" | "mensaje") {
        modal::exigir_habilitada(h, modal::PARA_ACTUAR)?;
    }
    let cerrar = action == "cerrar";
    // The pid must be read while the window still exists: a destroyed HWND
    // reports pid 0 and the re-observation would see nothing.
    let pid = cerrar.then(|| pid_of(h));
    let antes = pid.map(ventanas_vivas);
    ffi::traer_al_frente(ffi::get_ancestor(h, GA_ROOT));
    ejecutar_accion(h, action, args)?;

    let mut reabiertas = Vec::new();
    if cerrar {
        exigir_cerrada(h)?;
        std::thread::sleep(ESPERA_REAPARICION);
        if let (Some(antes), Some(pid)) = (antes, pid) {
            reabiertas = formas_reabiertas(&antes, &ventanas_vivas(pid), h);
        }
    }
    let mut window = window_json(h);
    if cerrar {
        if let Some(map) = window.as_object_mut() {
            map.insert("closed".into(), json!(true));
        }
    }
    Ok(json!({
        "invoked": reference,
        "hwnd": format!("0x{:X}", h),
        "action": action,
        "window": window,
        "reabiertas": reabiertas,
        "aviso": aviso_de_reaparicion(&reabiertas),
    }))
}

/// Closing a window that is already gone usually means PVsyst closed it by itself
/// (the trial dialog); it stays an error so a mistyped name is not taken for success.
fn no_encontrado(referencia: &str, accion: &str) -> Error {
    if accion == "cerrar" {
        return Error::Gui(format!("Window '{referencia}' not found (already closed?)"));
    }
    Error::Gui(format!("Control '{referencia}' not found"))
}

/// A close that left the window open (it asked for a confirmation, or a modal
/// intercepted it) is a failure the caller must see.
fn exigir_cerrada(h: Hwnd) -> Result<()> {
    if esperar_cierre(h, ESPERA_CIERRE) {
        return Ok(());
    }
    Err(Error::Gui(format!(
        "{} is still open after asking it to close: it may have opened a confirmation (gui_observar que:ventanas)",
        class_of(h)
    )))
}

#[cfg(test)]
#[path = "../../../tests/unit/adapters/gui/acciones.rs"]
mod tests;
