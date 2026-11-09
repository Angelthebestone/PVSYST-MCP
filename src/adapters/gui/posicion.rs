//! Where a control sits on its form, in the one coordinate system `gui_observar
//! que:vista` publishes and `gui_actuar campo` reads: px at 96 DPI from the
//! top-left corner of the window rectangle (title bar and border included).

use super::ffi::{Hwnd, Rect};

/// How far (px at 96 DPI) a requested point may be from a control's edge.
const TOLERANCIA_PX: i32 = 6;

/// Position of `r` on the form laid out at 96 DPI. A DPI-unaware window reports
/// physical rectangles for that layout, so `escala` divides them back. O(1).
pub(crate) fn posicion_de_diseno(ventana: Rect, r: Rect, escala: f64) -> (i32, i32) {
    let a_diseno = |v: i32| (f64::from(v) / escala).round() as i32;
    (
        a_diseno(r.left - ventana.left),
        a_diseno(r.top - ventana.top),
    )
}

/// Distance (px at 96 DPI) from `punto` to the span `[desde, hasta]`; 0 inside it.
fn fuera_de(punto: i32, desde: i32, hasta: i32) -> i32 {
    (desde - punto).max(punto - hasta).max(0)
}

/// The candidate whose rectangle holds the point `(x, y)` of the form, as
/// published by `vista`, or else the nearest one within [`TOLERANCIA_PX`]:
/// neighbouring fields are closer together than the tolerance. O(candidatos).
pub(crate) fn control_en(
    (ventana, escala): (Rect, f64),
    (x, y): (i32, i32),
    candidatos: impl IntoIterator<Item = (Hwnd, Rect)>,
) -> Option<Hwnd> {
    candidatos
        .into_iter()
        .map(|(h, r)| {
            let (izquierda, arriba) = posicion_de_diseno(ventana, r, escala);
            let (derecha, abajo) = posicion_de_diseno(
                ventana,
                Rect {
                    left: r.right,
                    top: r.bottom,
                    ..r
                },
                escala,
            );
            let distancia = fuera_de(x, izquierda, derecha).max(fuera_de(y, arriba, abajo));
            (h, distancia)
        })
        .filter(|(_, distancia)| *distancia <= TOLERANCIA_PX)
        .min_by_key(|(_, distancia)| *distancia)
        .map(|(h, _)| h)
}

#[cfg(test)]
#[path = "../../../tests/unit/adapters/gui/posicion.rs"]
mod tests;
