//! Text recognition of a window: a live capture, or a PNG saved earlier.

use super::ampliacion::{ampliar, factor_para, ImagenBgra, Region, LADO_MAXIMO_OCR};
use super::dominio::{LineaOcr, PalabraOcr};
use super::motor::{reconocer_texto, EntradaOcr};
use crate::adapters::gui::ffi::{self, Hwnd, Rect};
use crate::adapters::gui::gdi;
use crate::adapters::gui::ventanas::rect_of;
use crate::error::{Error, Result};
use serde_json::Value;

fn trasladar_linea(l: LineaOcr, offset_x: i32, offset_y: i32) -> LineaOcr {
    let mover = |r: Rect| Rect {
        left: r.left + offset_x,
        top: r.top + offset_y,
        right: r.right + offset_x,
        bottom: r.bottom + offset_y,
    };
    LineaOcr {
        texto: l.texto,
        rect: mover(l.rect),
        palabras: l
            .palabras
            .into_iter()
            .map(|p| PalabraOcr {
                texto: p.texto,
                rect: mover(p.rect),
            })
            .collect(),
    }
}

/// What to read: where saved PNGs resolve and which window is on screen.
pub(crate) struct ParametrosCaptura<'a> {
    pub(crate) paths: &'a crate::config::Paths,
    pub(crate) args: &'a Value,
    pub(crate) rect_win: &'a Rect,
}

/// Lines of text of window `h` in screen pixels: from the PNG in `args.ruta`
/// when given, else from a live capture.
pub(crate) fn capturar_lineas_pantalla(p: &ParametrosCaptura, h: Hwnd) -> Result<Vec<LineaOcr>> {
    let Some(ruta) = p.args.get("ruta").and_then(Value::as_str) else {
        return lineas_vivas(h);
    };
    // A saved PNG is already in whatever scale its author chose.
    Ok(reconocer_png_guardado(p, ruta)?
        .into_iter()
        .map(|l| trasladar_linea(l, p.rect_win.left, p.rect_win.top))
        .collect())
}

/// Lines of text of window `h` from a live capture, in screen pixels.
pub(crate) fn lineas_vivas(h: Hwnd) -> Result<Vec<LineaOcr>> {
    let r = rect_of(h);
    let dimensiones = ((r.right - r.left).max(1), (r.bottom - r.top).max(1));
    Ok(reconocer_ventana_viva(h, dimensiones)?
        .into_iter()
        .map(|l| trasladar_linea(l, r.left, r.top))
        .collect())
}

fn reconocer_png_guardado(p: &ParametrosCaptura, ruta: &str) -> Result<Vec<LineaOcr>> {
    let ruta = crate::workspace::resolver_ruta(
        &p.paths.workspace,
        ruta,
        crate::workspace::Alcance::de_args(p.args),
    )?;
    let bytes = std::fs::read(&ruta)
        .map_err(|e| Error::io(format!("Cannot read {}", ruta.display()), e))?;
    reconocer_texto(EntradaOcr::Png(&bytes))
}

/// OCR of a live capture, in window pixels. A DPI-unaware window (PVsyst)
/// renders its 96-DPI layout in the top-left of a bitmap as large as the
/// physical window: only that area is enlarged and read, and the lines come
/// back scaled to physical pixels.
fn reconocer_ventana_viva(h: Hwnd, (w, alto): (i32, i32)) -> Result<Vec<LineaOcr>> {
    let escala = ffi::escala_de_captura(h);
    let region = Region {
        ancho: ((f64::from(w) / escala) as u32).max(1),
        alto: ((f64::from(alto) / escala) as u32).max(1),
    };
    let factor = factor_para(region).ok_or_else(|| {
        Error::Gui(format!(
            "La ventana mide {}x{} px a 96 DPI: el OCR de Windows no acepta imágenes de más de {LADO_MAXIMO_OCR} px de lado",
            region.ancho, region.alto
        ))
    })?;
    let pixeles = gdi::capture_pixels(h, (w, alto))?;
    let origen = ImagenBgra {
        pixeles: &pixeles,
        ancho: w as u32,
        alto: alto as u32,
    };
    let ampliada = ampliar(origen, region, factor);
    let lineas = reconocer_texto(EntradaOcr::Bgra {
        ancho: region.ancho * factor,
        alto: region.alto * factor,
        pixeles: &ampliada,
    })?;
    Ok(lineas
        .into_iter()
        .map(|l| l.escalada(escala / f64::from(factor)))
        .collect())
}
