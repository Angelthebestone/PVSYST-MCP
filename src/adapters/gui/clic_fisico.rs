//! Where a physical click goes: the centre of the target, a point measured from
//! its corner, or a screen point the caller names.

use super::ffi::{self, Hwnd};
use super::ventanas::rect_of;
use crate::args::opt_i32;
use crate::error::{Error, Result};
use crate::idioma::{Idioma, Texto};
use serde_json::Value;

/// `dx`/`dy` of the request: a point of the control in px at 96 DPI from its
/// top-left corner. Both or neither: half a point has no meaning.
pub(super) fn desplazamiento(args: &Value, idioma: Idioma) -> Result<Option<(i32, i32)>> {
    match (opt_i32(args, "dx")?, opt_i32(args, "dy")?) {
        (Some(dx), Some(dy)) => Ok(Some((dx, dy))),
        (None, None) => Ok(None),
        _ => {
            let juntos = Texto {
                es: "'dx' y 'dy' van juntos (enteros, px a 96 DPI desde la esquina del control)",
                en: "'dx' and 'dy' go together (integers, px at 96 DPI from the control corner)",
            };
            Err(Error::Invalid(juntos.en_idioma(idioma).into()))
        }
    }
}

/// Where on the target a physical click goes.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum PuntoPedido {
    Centro,
    /// px at 96 DPI from the top-left corner of the control (`dx`, `dy`).
    EnControl(i32, i32),
    /// A screen point (`objetivo:{x,y}`), such as the `punto` `clic_texto` answers.
    EnPantalla(i32, i32),
}

/// The point the request asks for: `objetivo` or `dx`/`dy`, never both.
pub(super) fn punto_pedido(args: &Value, idioma: Idioma) -> Result<PuntoPedido> {
    let Some(objetivo) = args.get("objetivo") else {
        return Ok(
            desplazamiento(args, idioma)?.map_or(PuntoPedido::Centro, |(dx, dy)| {
                PuntoPedido::EnControl(dx, dy)
            }),
        );
    };
    if desplazamiento(args, idioma)?.is_some() {
        let dos_formas = Texto {
            es: "'objetivo' y 'dx'/'dy' son dos formas de elegir el punto: pasa solo una",
            en: "'objetivo' and 'dx'/'dy' are two ways of choosing the point: pass only one",
        };
        return Err(Error::Invalid(dos_formas.en_idioma(idioma).into()));
    }
    let coordenada = |clave: &str| {
        objetivo
            .get(clave)
            .and_then(Value::as_i64)
            .and_then(|v| i32::try_from(v).ok())
            .ok_or_else(|| {
                let falta = Texto {
                    es: "'objetivo' de clic_fisico es {x, y} en píxeles de pantalla; falta o no es entero '{clave}'",
                    en: "'objetivo' of clic_fisico is {x, y} in screen pixels: '{clave}' is missing or not an integer",
                };
                Error::Invalid(falta.texto_con(idioma, &[("clave", clave)]))
            })
    };
    Ok(PuntoPedido::EnPantalla(coordenada("x")?, coordenada("y")?))
}

/// Screen point a physical click on window `r` goes to. A requested screen point
/// must lie on the window: the desktop is shared.
fn punto_de_clic_fisico(h: Hwnd, pedido: PuntoPedido, idioma: Idioma) -> Result<(i32, i32)> {
    let (r, dpi) = (rect_of(h), ffi::dpi_de(h));
    match pedido {
        PuntoPedido::Centro => Ok(((r.left + r.right) / 2, (r.top + r.bottom) / 2)),
        PuntoPedido::EnControl(dx, dy) => Ok((
            r.left + ffi::escalar_a_dpi(dx, dpi),
            r.top + ffi::escalar_a_dpi(dy, dpi),
        )),
        PuntoPedido::EnPantalla(x, y)
            if (r.left..r.right).contains(&x) && (r.top..r.bottom).contains(&y) =>
        {
            Ok((x, y))
        }
        PuntoPedido::EnPantalla(x, y) => {
            let fuera = Texto {
                es: "El punto de pantalla ({x}, {y}) no está sobre la ventana ({izquierda}, {arriba})-({derecha}, {abajo}): el escritorio es compartido y no se pulsa fuera de ella",
                en: "The screen point ({x}, {y}) is not on the window ({izquierda}, {arriba})-({derecha}, {abajo}): the desktop is shared and nothing is clicked outside it",
            };
            Err(Error::Invalid(fuera.texto_con(
                idioma,
                &[
                    ("x", &x.to_string()),
                    ("y", &y.to_string()),
                    ("izquierda", &r.left.to_string()),
                    ("arriba", &r.top.to_string()),
                    ("derecha", &r.right.to_string()),
                    ("abajo", &r.bottom.to_string()),
                ],
            )))
        }
    }
}

pub(crate) fn clic_fisico_en_control(h: Hwnd, pedido: PuntoPedido, idioma: Idioma) -> Result<()> {
    let r = rect_of(h);
    if r.right <= r.left {
        let sin_rectangulo = Texto {
            es: "El control 0x{hwnd} no tiene rectángulo",
            en: "Control 0x{hwnd} has no rectangle",
        };
        return Err(Error::Gui(
            sin_rectangulo.texto_con(idioma, &[("hwnd", &format!("{h:X}"))]),
        ));
    }
    let (x, y) = punto_de_clic_fisico(h, pedido, idioma)?;
    ffi::clic_fisico_en(x, y);
    Ok(())
}

#[cfg(test)]
#[path = "../../../tests/unit/adapters/gui/clic_fisico.rs"]
mod tests;
