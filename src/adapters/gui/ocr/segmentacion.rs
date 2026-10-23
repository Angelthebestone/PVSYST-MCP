//! Splits OCR lines into caption segments. Windows OCR joins everything on one
//! text row into a single line, so a caption, its field's own text and the
//! caption of the next column come back glued together. Pure: no I/O.

use super::dominio::{LineaOcr, PalabraOcr};
use crate::adapters::gui::ffi::Rect;

/// Margin that still counts a word as drawn inside a control.
const MARGEN_CONTROL_PX: i32 = 2;

/// Words of one caption and the box that holds them.
struct Segmento {
    caja: Rect,
    palabras: Vec<PalabraOcr>,
}

impl Segmento {
    fn con(palabra: &PalabraOcr) -> Self {
        Self {
            caja: palabra.rect,
            palabras: vec![palabra.clone()],
        }
    }

    fn sumar(&mut self, palabra: &PalabraOcr) {
        self.caja = self.caja.union(&palabra.rect);
        self.palabras.push(palabra.clone());
    }

    fn a_linea(self) -> LineaOcr {
        let texto = self
            .palabras
            .iter()
            .map(|p| p.texto.as_str())
            .collect::<Vec<_>>()
            .join(" ");
        LineaOcr {
            texto,
            rect: self.caja,
            palabras: self.palabras,
        }
    }
}

/// Caption segments of `lineas`: words centred inside any of `controles` are
/// the control's own text and are dropped, and a gap wider than the word's
/// height starts a new segment. O(palabras · controles).
pub(crate) fn segmentar_lineas(lineas: &[LineaOcr], controles: &[Rect]) -> Vec<LineaOcr> {
    lineas
        .iter()
        .flat_map(|linea| segmentos_de(&linea.palabras, controles))
        .map(Segmento::a_linea)
        .collect()
}

fn segmentos_de(palabras: &[PalabraOcr], controles: &[Rect]) -> Vec<Segmento> {
    palabras
        .iter()
        .filter(|p| !dentro_de_un_control(&p.rect, controles))
        .fold(Vec::new(), agrupar)
}

fn dentro_de_un_control(palabra: &Rect, controles: &[Rect]) -> bool {
    let (x, y) = ((palabra.left + palabra.right) / 2, palabra.centro_y());
    controles.iter().any(|c| {
        x >= c.left - MARGEN_CONTROL_PX
            && x <= c.right + MARGEN_CONTROL_PX
            && y >= c.top - MARGEN_CONTROL_PX
            && y <= c.bottom + MARGEN_CONTROL_PX
    })
}

fn agrupar(mut grupos: Vec<Segmento>, palabra: &PalabraOcr) -> Vec<Segmento> {
    let continua = grupos
        .last()
        .is_some_and(|g| palabra.rect.left - g.caja.right <= alto_de(&palabra.rect));
    match grupos.last_mut() {
        Some(ultimo) if continua => ultimo.sumar(palabra),
        _ => grupos.push(Segmento::con(palabra)),
    }
    grupos
}

fn alto_de(r: &Rect) -> i32 {
    r.bottom - r.top
}

#[cfg(test)]
#[path = "../../../../tests/unit/adapters/gui/ocr/segmentacion.rs"]
mod tests;
