//! Where a physical click goes: the centre of the target, a point measured from
//! its corner, or a screen point the caller names.

use super::ffi::{self, Hwnd};
use super::ventanas::rect_of;
use crate::args::opt_i32;
use crate::error::{Error, Result};
use serde_json::Value;

/// `dx`/`dy` of the request: a point of the control in px at 96 DPI from its
/// top-left corner. Both or neither: half a point has no meaning.
pub(super) fn desplazamiento(args: &Value) -> Result<Option<(i32, i32)>> {
    match (opt_i32(args, "dx")?, opt_i32(args, "dy")?) {
        (Some(dx), Some(dy)) => Ok(Some((dx, dy))),
        (None, None) => Ok(None),
        _ => Err(Error::Invalid(
            "'dx' and 'dy' go together (integers, px at 96 DPI from the control corner)".into(),
        )),
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
pub(super) fn punto_pedido(args: &Value) -> Result<PuntoPedido> {
    let Some(objetivo) = args.get("objetivo") else {
        return Ok(
            desplazamiento(args)?.map_or(PuntoPedido::Centro, |(dx, dy)| {
                PuntoPedido::EnControl(dx, dy)
            }),
        );
    };
    if desplazamiento(args)?.is_some() {
        return Err(Error::Invalid(
            "'objetivo' y 'dx'/'dy' son dos formas de elegir el punto: pasa solo una".into(),
        ));
    }
    let coordenada = |clave: &str| {
        objetivo
            .get(clave)
            .and_then(Value::as_i64)
            .and_then(|v| i32::try_from(v).ok())
            .ok_or_else(|| {
                Error::Invalid(format!(
                    "'objetivo' de clic_fisico es {{x, y}} en pixeles de pantalla; falta o no es entero '{clave}'"
                ))
            })
    };
    Ok(PuntoPedido::EnPantalla(coordenada("x")?, coordenada("y")?))
}

/// Screen point a physical click on window `r` goes to. A requested screen point
/// must lie on the window: the desktop is shared.
fn punto_de_clic_fisico(r: ffi::Rect, dpi: u32, pedido: PuntoPedido) -> Result<(i32, i32)> {
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
        PuntoPedido::EnPantalla(x, y) => Err(Error::Invalid(format!(
            "El punto de pantalla ({x}, {y}) no está sobre la ventana ({}, {})-({}, {}): el escritorio es compartido y no se pulsa fuera de ella",
            r.left, r.top, r.right, r.bottom
        ))),
    }
}

pub(crate) fn clic_fisico_en_control(h: Hwnd, pedido: PuntoPedido) -> Result<()> {
    let r = rect_of(h);
    if r.right <= r.left {
        return Err(Error::Gui(format!("Control 0x{h:X} has no rectangle")));
    }
    let (x, y) = punto_de_clic_fisico(r, ffi::dpi_de(h), pedido)?;
    ffi::clic_fisico_en(x, y);
    Ok(())
}

#[cfg(test)]
#[path = "../../../tests/unit/adapters/gui/clic_fisico.rs"]
mod tests;
