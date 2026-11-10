//! Window actions: clicks, messages, physical inputs and desktop interactivity.

use super::cierre;
use super::clic_fisico::{clic_fisico_en_control, desplazamiento, punto_pedido};
use super::ffi::{self, Hwnd, BM_CLICK, GA_ROOT, SW_RESTORE, WM_CLOSE};
use super::modal;
use super::rotulos::{rotulos_acciones, Idioma, RotulosAcciones};
use super::ventanas::{class_of, pid_of, resolve, text_of, ventanas_de, visible, window_json};
use crate::args::{opt_u32, req_str};
use crate::error::{Error, Result};
use crate::idioma::{Idioma as IdiomaCliente, Texto};
use serde_json::{json, Value};

/// Window classes (Win32 and VCL) that implement `BM_CLICK`.
const CLASES_BOTON: &[&str] = &["Button", "TButton", "TBitBtn", "TCheckBox", "TRadioButton"];
/// Buttons that abandon a form: tried first when the request says `cancelar:true`.
const BOTONES_CANCELAR: &[&str] = &["Cancelar", "No"];
const ROTULOS: RotulosAcciones = rotulos_acciones(Idioma::Es);
/// Class of PVsyst's message boxes.
pub(crate) const CLASE_MENSAJE: &str = "TMessageForm";
/// Class of the madExcept crash dialog.
pub(crate) const CLASE_EXCEPCION: &str = "madExceptWndClass";

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

fn ejecutar_accion(h: Hwnd, action: &str, args: &Value, idioma: IdiomaCliente) -> Result<()> {
    match action {
        "click" => match desplazamiento(args, idioma)? {
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
        "clic_fisico" => clic_fisico_en_control(h, punto_pedido(args, idioma)?, idioma)?,
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

/// `texto` and `id` name no control for these actions: they are for `clic_texto`
/// and `menu`. Accepting them silently clicked the window instead (GUI-06).
fn exigir_sin_texto_ni_id(args: &Value, idioma: IdiomaCliente) -> Result<()> {
    for clave in ["texto", "id"] {
        if args.get(clave).is_some() {
            let no_selecciona = Texto {
                es: "'{clave}' no selecciona el control en click/clic_fisico/cerrar/activar/mensaje: nómbralo con 'ventana' (hwnd, clase o título) o pulsa texto visible con accion:clic_texto",
                en: "'{clave}' does not select the control in click/clic_fisico/cerrar/activar/mensaje: name it with 'ventana' (hwnd, class or title) or click visible text with accion:clic_texto",
            };
            return Err(Error::Invalid(
                no_selecciona.texto_con(idioma, &[("clave", clave)]),
            ));
        }
    }
    Ok(())
}

/// A disabled control swallows the click without a trace (GUI-07).
fn exigir_control_habilitado(
    habilitada: bool,
    clase: &str,
    texto: &str,
    idioma: IdiomaCliente,
) -> Result<()> {
    if habilitada {
        return Ok(());
    }
    let deshabilitado = Texto {
        es: "El control {clase} '{texto}' está deshabilitado: el clic no haría nada. Habilita lo que lo condiciona (gui_observar que:vista) y repite",
        en: "The {clase} control '{texto}' is disabled: the click would do nothing. Enable what conditions it (gui_observar que:vista) and retry",
    };
    Err(Error::Gui(
        deshabilitado.texto_con(idioma, &[("clase", clase), ("texto", texto)]),
    ))
}

pub(crate) fn invocar(args: &Value, idioma: IdiomaCliente) -> Result<Value> {
    exigir_sin_texto_ni_id(args, idioma)?;
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
    if matches!(action, "click" | "clic_fisico") {
        exigir_control_habilitado(
            ffi::estado_ventana(h).habilitada,
            &class_of(h),
            &text_of(h),
            idioma,
        )?;
    }
    let cerrar = action == "cerrar";
    // The pid must be read while the window still exists: a destroyed HWND
    // reports pid 0 and the re-observation would see nothing.
    let pid = cerrar.then(|| pid_of(h));
    let antes = pid.map(cierre::instantanea);
    ffi::traer_al_frente(ffi::get_ancestor(h, GA_ROOT));
    ejecutar_accion(h, action, args, idioma)?;

    let mut reabiertas = Vec::new();
    if cerrar {
        cierre::exigir_cerrada(h, idioma)?;
        std::thread::sleep(cierre::ESPERA_REAPARICION);
        if let (Some(antes), Some(pid)) = (antes, pid) {
            reabiertas = cierre::reabiertas(&antes, &cierre::instantanea(pid), h);
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
        "aviso": cierre::aviso(&reabiertas),
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

#[cfg(test)]
#[path = "../../../tests/unit/adapters/gui/acciones.rs"]
mod tests;
