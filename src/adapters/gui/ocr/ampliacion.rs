//! Enlarges captured pixels so Windows OCR can read the small captions of a
//! 96-DPI VCL form. Pure: no Win32, no WinRT.

const BYTES_POR_PIXEL: usize = 4;
/// Channels that carry colour; the fourth one is alpha and stays untouched.
const CANALES_DE_COLOR: usize = 3;
/// Weight of each of the four neighbours the sharpening kernel subtracts.
const PESO_VECINO: f32 = 0.5;
const VECINOS: f32 = 4.0;
/// Longest image side `Windows.Media.Ocr` accepts (`OcrEngine.MaxImageDimension`).
pub(crate) const LADO_MAXIMO_OCR: u32 = 2600;
/// Beyond this factor the interpolated pixels stop helping the recognizer.
const FACTOR_MAXIMO: u32 = 3;

/// Size in pixels of the area to enlarge, anchored at the image's top-left.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct Region {
    pub(crate) ancho: u32,
    pub(crate) alto: u32,
}

/// Borrowed top-down BGRA pixels with the size they were captured at.
#[derive(Debug, Clone, Copy)]
pub(crate) struct ImagenBgra<'a> {
    pub(crate) pixeles: &'a [u8],
    pub(crate) ancho: u32,
    pub(crate) alto: u32,
}

/// How a source `region` is enlarged.
#[derive(Debug, Clone, Copy)]
struct Muestreo {
    region: Region,
    factor: u32,
}

/// Largest integer factor (at most [`FACTOR_MAXIMO`]) that keeps the longest
/// side of `region` within [`LADO_MAXIMO_OCR`]; `None` when the region is
/// already larger than the recognizer accepts, whatever the factor.
pub(crate) fn factor_para(region: Region) -> Option<u32> {
    let lado = region.ancho.max(region.alto).max(1);
    match LADO_MAXIMO_OCR / lado {
        0 => None,
        factor => Some(factor.min(FACTOR_MAXIMO)),
    }
}

/// Crops `region` from the top-left of `origen`, enlarges it `factor` times
/// with bilinear interpolation and sharpens the result: interpolation blurs
/// glyph edges enough for the recognizer to confuse letters otherwise.
/// O(ancho·alto·factor²) time; neighbour tables are computed once per axis so
/// the per-pixel work is only the blend. `region` must lie inside the image.
pub(crate) fn ampliar(origen: ImagenBgra<'_>, region: Region, factor: u32) -> Vec<u8> {
    let (ancho, alto) = (region.ancho * factor, region.alto * factor);
    let ampliada = ampliar_bilineal(origen, Muestreo { region, factor });
    afilar(&ampliada, ancho, alto)
}

/// Source pixels on both sides of one output coordinate and the weight of the
/// second one.
type Vecinos = (usize, usize, f32);

fn tabla_de_vecinos(salida: u32, factor: u32, limite: u32) -> Vec<Vecinos> {
    (0..salida)
        .map(|o| {
            let (a, b, peso) = vecinos(o, factor, limite);
            (a as usize, b as usize, peso)
        })
        .collect()
}

fn ampliar_bilineal(origen: ImagenBgra<'_>, muestreo: Muestreo) -> Vec<u8> {
    let Muestreo { region, factor } = muestreo;
    let (ancho, alto) = (region.ancho * factor, region.alto * factor);
    let columnas = tabla_de_vecinos(ancho, factor, region.ancho);
    let filas = tabla_de_vecinos(alto, factor, region.alto);
    let mut ampliada = Vec::with_capacity(ancho as usize * alto as usize * BYTES_POR_PIXEL);
    for &(y0, y1, fy) in &filas {
        let pares = (fila_de(origen, y0), fila_de(origen, y1));
        for &columna in &columnas {
            ampliada.extend(pixel_bilineal(pares, columna, fy));
        }
    }
    ampliada
}

fn fila_de<'a>(imagen: ImagenBgra<'a>, y: usize) -> &'a [u8] {
    if y >= imagen.alto as usize {
        return &[];
    }
    let largo = imagen.ancho as usize * BYTES_POR_PIXEL;
    imagen
        .pixeles
        .get(y * largo..(y + 1) * largo)
        .unwrap_or(&[])
}

fn canal_de(fila: &[u8], x: usize, canal: usize) -> f32 {
    fila.get(x * BYTES_POR_PIXEL + canal)
        .copied()
        .map_or(0.0, f32::from)
}

fn pixel_bilineal(
    (arriba, abajo): (&[u8], &[u8]),
    (x0, x1, fx): Vecinos,
    fy: f32,
) -> [u8; BYTES_POR_PIXEL] {
    std::array::from_fn(|canal| {
        let sobre = mezclar(canal_de(arriba, x0, canal), canal_de(arriba, x1, canal), fx);
        let bajo = mezclar(canal_de(abajo, x0, canal), canal_de(abajo, x1, canal), fx);
        mezclar(sobre, bajo, fy).round() as u8
    })
}

/// Source pixels on both sides of output coordinate `o` and the weight of the
/// second one.
fn vecinos(o: u32, factor: u32, limite: u32) -> (u32, u32, f32) {
    let ultimo = limite.saturating_sub(1);
    let centro = ((o as f32 + 0.5) / factor as f32 - 0.5).max(0.0);
    let base = (centro.floor() as u32).min(ultimo);
    let peso = (centro - base as f32).clamp(0.0, 1.0);
    (base, (base + 1).min(ultimo), peso)
}

fn mezclar(a: f32, b: f32, peso_de_b: f32) -> f32 {
    a + (b - a) * peso_de_b
}

/// 3×3 sharpening `v·(1 + 4k) − k·(arriba + abajo + izquierda + derecha)` per
/// colour channel; neighbours outside the image repeat the edge pixel.
fn afilar(imagen: &[u8], ancho: u32, alto: u32) -> Vec<u8> {
    let origen = ImagenBgra {
        pixeles: imagen,
        ancho,
        alto,
    };
    let (ancho, alto) = (ancho as usize, alto as usize);
    let mut afilada = Vec::with_capacity(imagen.len());
    for y in 0..alto {
        let filas = (
            fila_de(origen, y.saturating_sub(1)),
            fila_de(origen, y),
            fila_de(origen, (y + 1).min(alto.saturating_sub(1))),
        );
        for x in 0..ancho {
            let lados = (x.saturating_sub(1), (x + 1).min(ancho.saturating_sub(1)));
            afilada.extend(pixel_afilado(filas, x, lados));
        }
    }
    afilada
}

fn pixel_afilado(
    (arriba, fila, abajo): (&[u8], &[u8], &[u8]),
    x: usize,
    (izquierda, derecha): (usize, usize),
) -> [u8; BYTES_POR_PIXEL] {
    std::array::from_fn(|canal| {
        let valor = canal_de(fila, x, canal);
        if canal >= CANALES_DE_COLOR {
            return valor as u8;
        }
        let suma = canal_de(arriba, x, canal)
            + canal_de(abajo, x, canal)
            + canal_de(fila, izquierda, canal)
            + canal_de(fila, derecha, canal);
        (valor * (1.0 + VECINOS * PESO_VECINO) - PESO_VECINO * suma)
            .round()
            .clamp(0.0, 255.0) as u8
    })
}

#[cfg(test)]
#[path = "../../../../tests/unit/adapters/gui/ocr/ampliacion.rs"]
mod tests;
