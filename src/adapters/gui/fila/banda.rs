//! Finds the selected row in two captures (before and after the click) without
//! reading any text: the selection paints most of the width of its row with a
//! colour the background does not have, and the click is what changed it.

use super::geometria::Geometria;
use crate::adapters::gui::captura::ImagenRgb;
use std::collections::HashMap;

const BYTES_POR_PIXEL: usize = 3;
/// Share of a row line (in %) one colour must fill to count as the row colour.
const UMBRAL_BANDA_PCT: usize = 50;

type Color = [u8; BYTES_POR_PIXEL];
/// Dominant colour of a row line and the % of the line it fills.
type Muestra = Option<(Color, usize)>;

/// Most frequent colour of `linea` and the % of the line it fills. O(n). A tie
/// goes to the larger colour so the answer never depends on hash order.
fn dominante(linea: &[u8]) -> Muestra {
    let pixeles = linea.as_chunks::<BYTES_POR_PIXEL>().0;
    let mut cuentas: HashMap<Color, usize> = HashMap::new();
    for px in pixeles {
        *cuentas.entry(*px).or_default() += 1;
    }
    let (color, n) = cuentas.into_iter().max_by_key(|(c, n)| (*n, *c))?;
    Some((color, n * 100 / pixeles.len()))
}

fn linea_en(img: &ImagenRgb, y: i32) -> Option<&[u8]> {
    let y = usize::try_from(y).ok().filter(|y| *y < img.alto)?;
    let ancho = img.ancho * BYTES_POR_PIXEL;
    img.rgb.get(y * ancho..(y + 1) * ancho)
}

/// One sample per row slot of the control.
fn muestras(img: &ImagenRgb, geom: &Geometria) -> Vec<Muestra> {
    (0..geom.filas_visibles(img.alto_px()))
        .map(|n| linea_en(img, geom.centro_de_fila(n)).and_then(dominante))
        .collect()
}

/// Background colour: the one most row slots are painted with.
fn fondo_de(muestras: &[Muestra]) -> Option<Color> {
    let mut cuentas: HashMap<Color, usize> = HashMap::new();
    for (color, _) in muestras.iter().flatten() {
        *cuentas.entry(*color).or_default() += 1;
    }
    cuentas
        .into_iter()
        .max_by_key(|(c, n)| (*n, *c))
        .map(|(c, _)| c)
}

/// Row slots painted with a colour other than the background.
fn filas_coloreadas(muestras: &[Muestra]) -> Vec<usize> {
    let Some(fondo) = fondo_de(muestras) else {
        return Vec::new();
    };
    let resaltada =
        |m: &Muestra| matches!(m, Some((c, pct)) if *c != fondo && *pct >= UMBRAL_BANDA_PCT);
    (0..muestras.len())
        .filter(|n| muestras.get(*n).is_some_and(resaltada))
        .collect()
}

/// The two captures of the same control, taken around the click.
pub(crate) struct Capturas<'a> {
    pub(crate) antes: &'a ImagenRgb,
    pub(crate) despues: &'a ImagenRgb,
}

/// Row slots the click selected. A row painted with a status colour is coloured in
/// both captures, so when something changed only the rows that changed count; when
/// nothing changed, the row was already selected and the coloured rows are the answer.
pub(crate) fn filas_seleccionadas(capturas: &Capturas<'_>, geom: &Geometria) -> Vec<usize> {
    let antes = muestras(capturas.antes, geom);
    let despues = muestras(capturas.despues, geom);
    let coloreadas = filas_coloreadas(&despues);
    let cambiadas: Vec<usize> = (0..despues.len())
        .filter(|n| antes.get(*n) != despues.get(*n))
        .collect();
    if cambiadas.is_empty() {
        return coloreadas;
    }
    coloreadas
        .into_iter()
        .filter(|n| cambiadas.contains(n))
        .collect()
}

#[cfg(test)]
#[path = "../../../../tests/unit/adapters/gui/fila/banda.rs"]
mod tests;
