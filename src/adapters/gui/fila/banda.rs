//! Finds the highlighted (selected) row in a capture without reading any text:
//! the selection paints most of the width of its row with a colour the other
//! rows do not have.

use super::geometria::Geometria;
use crate::adapters::gui::captura::ImagenRgb;
use std::collections::HashMap;

const BYTES_POR_PIXEL: usize = 3;
/// Share of a row line (in %) one colour must fill to count as the row colour.
const UMBRAL_BANDA_PCT: usize = 50;

type Color = [u8; BYTES_POR_PIXEL];

/// Most frequent colour of `linea` and the % of the line it fills. O(n).
fn dominante(linea: &[u8]) -> Option<(Color, usize)> {
    let pixeles = linea.as_chunks::<BYTES_POR_PIXEL>().0;
    let mut cuentas: HashMap<Color, usize> = HashMap::new();
    for px in pixeles {
        *cuentas.entry(*px).or_default() += 1;
    }
    let (color, n) = cuentas.into_iter().max_by_key(|(_, n)| *n)?;
    Some((color, n * 100 / pixeles.len()))
}

fn linea_en(img: &ImagenRgb, y: i32) -> Option<&[u8]> {
    let y = usize::try_from(y).ok().filter(|y| *y < img.alto)?;
    let ancho = img.ancho * BYTES_POR_PIXEL;
    img.rgb.get(y * ancho..(y + 1) * ancho)
}

/// Background colour: the one most row slots are painted with.
fn fondo_de(muestras: &[Option<(Color, usize)>]) -> Option<Color> {
    let mut cuentas: HashMap<Color, usize> = HashMap::new();
    for (color, _) in muestras.iter().flatten() {
        *cuentas.entry(*color).or_default() += 1;
    }
    cuentas.into_iter().max_by_key(|(_, n)| *n).map(|(c, _)| c)
}

/// Row slots of `img` painted with the selection colour, in order.
pub(crate) fn filas_resaltadas(img: &ImagenRgb, geom: &Geometria) -> Vec<usize> {
    let muestras: Vec<_> = (0..geom.filas_visibles(img.alto_px()))
        .map(|n| linea_en(img, geom.centro_de_fila(n)).and_then(dominante))
        .collect();
    let Some(fondo) = fondo_de(&muestras) else {
        return Vec::new();
    };
    let resaltada = |m: &Option<(Color, usize)>| matches!(m, Some((c, pct)) if *c != fondo && *pct >= UMBRAL_BANDA_PCT);
    (0..muestras.len())
        .filter(|n| muestras.get(*n).is_some_and(resaltada))
        .collect()
}

#[cfg(test)]
#[path = "../../../../tests/unit/adapters/gui/fila/banda.rs"]
mod tests;
