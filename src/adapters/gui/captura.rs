//! GDI screen capture workflow, image cropping and PNG file writing.

use super::base64::base64;
use super::ffi::{self, Hwnd, GA_ROOT};
use super::gdi::capture_pixels;
use super::png::png_rgb;
use super::ventanas::{class_of, hwnd_de, json_identidad_ventana, rect_of, visible};
use crate::args::flag;
use crate::error::{Error, Result};
use crate::workspace::{resolver_ruta, Alcance};
use serde_json::{json, Value};
use std::path::PathBuf;

const BYTES_PER_PIXEL_32: usize = 4;
const BYTES_PER_PIXEL_RGB: usize = 3;

/// Converts top-down BGRX pixels to packed RGB.
pub(crate) fn rgb_desde_bgrx(px: &[u8]) -> Vec<u8> {
    px.as_chunks::<BYTES_PER_PIXEL_32>()
        .0
        .iter()
        .flat_map(|&[b, g, r, _]| [r, g, b])
        .collect()
}

pub(crate) struct DimensionesRecorte {
    pub(crate) ancho: usize,
    pub(crate) alto: usize,
    pub(crate) x: i32,
    pub(crate) y: i32,
    pub(crate) ancho_corte: i32,
    pub(crate) alto_corte: i32,
}

/// Crops packed RGB (`w` x `h`) to the rectangle at (`x`, `y`) of size
/// `cw` x `ch`, clamped to the image. Returns the pixels and the final size.
pub(crate) fn recortar_rgb(rgb: &[u8], dims: &DimensionesRecorte) -> (Vec<u8>, usize, usize) {
    if dims.ancho == 0 || dims.alto == 0 || dims.ancho_corte <= 0 || dims.alto_corte <= 0 {
        return (Vec::new(), 0, 0);
    }
    let max_x = (dims.ancho as i32 - 1).max(0);
    let max_y = (dims.alto as i32 - 1).max(0);
    let x = dims.x.clamp(0, max_x) as usize;
    let y = dims.y.clamp(0, max_y) as usize;
    let cw = (dims.ancho_corte as usize).min(dims.ancho.saturating_sub(x));
    let ch = (dims.alto_corte as usize).min(dims.alto.saturating_sub(y));
    let mut out = Vec::with_capacity(cw * ch * BYTES_PER_PIXEL_RGB);
    for fila in y..y + ch {
        let ini = (fila * dims.ancho + x) * BYTES_PER_PIXEL_RGB;
        let fin = ini.saturating_add(cw * BYTES_PER_PIXEL_RGB);
        if let Some(trozo) = rgb.get(ini..fin) {
            out.extend_from_slice(trozo);
        }
    }
    (out, cw, ch)
}

/// Output path for a capture: `ruta` (confined to workspace unless permitted)
/// or a temp file named after the control's class.
pub(crate) fn ruta_salida(paths: &crate::config::Paths, args: &Value, h: Hwnd) -> Result<PathBuf> {
    if let Some(r) = args.get("ruta").and_then(Value::as_str) {
        resolver_ruta(&paths.workspace, r, Alcance::de_args(args))
    } else {
        Ok(std::env::temp_dir().join(format!(
            "pvsyst_gui_{}.png",
            class_of(h).replace([':', '\\'], "_")
        )))
    }
}

/// Pixels a capture really holds. A DPI-unaware window (PVsyst) is drawn at
/// 96 DPI in the top-left of a bitmap as large as its physical size: the rest
/// stays black, so only `1/escala` of each side is the window.
#[derive(Debug, Clone, Copy)]
pub(crate) struct Lienzo {
    pub(crate) ancho: usize,
    pub(crate) alto: usize,
    pub(crate) escala: f64,
}

impl Lienzo {
    /// Drawn area of a `ancho` x `alto` capture at `escala` (>= 1). O(1).
    pub(crate) fn dibujado((ancho, alto): (i32, i32), escala: f64) -> Self {
        let lado = |v: i32| ((f64::from(v) / escala).round() as usize).max(1);
        Self {
            ancho: lado(ancho),
            alto: lado(alto),
            escala,
        }
    }

    /// A physical-pixel distance expressed in drawn pixels.
    fn a_dibujado(&self, v: i32) -> i32 {
        (f64::from(v) / self.escala).round() as i32
    }
}

fn aplicar_recorte(handles: (Hwnd, Hwnd), rgb: &[u8], lienzo: Lienzo) -> (Vec<u8>, usize, usize) {
    let (h, raiz) = handles;
    let rect = rect_of(h);
    let rect_raiz = rect_of(raiz);
    let dims = DimensionesRecorte {
        ancho: lienzo.ancho,
        alto: lienzo.alto,
        x: lienzo.a_dibujado(rect.left - rect_raiz.left),
        y: lienzo.a_dibujado(rect.top - rect_raiz.top),
        ancho_corte: lienzo.a_dibujado(rect.right - rect.left),
        alto_corte: lienzo.a_dibujado(rect.bottom - rect.top),
    };
    recortar_rgb(rgb, &dims)
}

/// Drops the black margin a DPI-unaware window leaves in its capture. `rgb` is
/// `ancho_fisico` pixels wide; it is returned as is when nothing is to drop.
fn solo_area_dibujada(rgb: Vec<u8>, ancho_fisico: usize, lienzo: Lienzo) -> Vec<u8> {
    if lienzo.ancho == ancho_fisico && rgb.len() == ancho_fisico * lienzo.alto * BYTES_PER_PIXEL_RGB
    {
        return rgb;
    }
    let alto_fisico = rgb.len() / (ancho_fisico * BYTES_PER_PIXEL_RGB).max(1);
    let dims = DimensionesRecorte {
        ancho: ancho_fisico,
        alto: alto_fisico,
        x: 0,
        y: 0,
        ancho_corte: lienzo.ancho as i32,
        alto_corte: lienzo.alto as i32,
    };
    recortar_rgb(&rgb, &dims).0
}

fn escribir_png(out: &PathBuf, png: &[u8]) -> Result<()> {
    if let Some(parent) = out.parent() {
        std::fs::create_dir_all(parent)
            .map_err(|e| Error::io(format!("Cannot create dir {}", parent.display()), e))?;
    }
    std::fs::write(out, png).map_err(|e| Error::io(format!("Cannot write {}", out.display()), e))
}

/// Capture a control (or its root window with `recorte:true`) to a PNG file.
pub(crate) fn capturar(paths: &crate::config::Paths, args: &Value) -> Result<Value> {
    let h = hwnd_de(args, "ventana")?;
    if !visible(h) {
        return Err(Error::Gui(format!(
            "0x{h:X} ({}) está oculto: no hay nada que capturar",
            class_of(h)
        )));
    }
    let recorte = flag(args, &["recorte"]);
    let raiz = if recorte {
        ffi::get_ancestor(h, GA_ROOT)
    } else {
        h
    };
    let rect_raiz = rect_of(raiz);
    let w = (rect_raiz.right - rect_raiz.left).max(1);
    let alto = (rect_raiz.bottom - rect_raiz.top).max(1);
    let rgb = rgb_desde_bgrx(&capture_pixels(raiz, (w, alto))?);
    if rgb.iter().all(|&b| b == 0) {
        return Err(Error::Gui(
            "Captura negra: escritorio no interactivo".into(),
        ));
    }
    let lienzo = Lienzo::dibujado((w, alto), ffi::escala_de_captura(raiz));
    let rgb_dibujado = solo_area_dibujada(rgb, w as usize, lienzo);
    let (rgb_final, ancho, alto_img) = if recorte {
        aplicar_recorte((h, raiz), &rgb_dibujado, lienzo)
    } else {
        (rgb_dibujado, lienzo.ancho, lienzo.alto)
    };
    if ancho == 0 || alto_img == 0 || rgb_final.is_empty() {
        return Err(Error::Gui(
            "El área de la captura está vacía (0x0 píxeles)".into(),
        ));
    }
    let png = png_rgb(ancho as u32, alto_img as u32, &rgb_final)?;
    let out = ruta_salida(paths, args, h)?;
    escribir_png(&out, &png)?;
    let mut obj = json_identidad_ventana(h);
    obj.insert("path".into(), json!(out.to_string_lossy()));
    obj.insert("width".into(), json!(ancho));
    obj.insert("height".into(), json!(alto_img));
    obj.insert("bytes".into(), json!(png.len()));
    if flag(args, &["imagen"]) {
        obj.insert("_imagen_png_base64".into(), json!(base64(&png)));
    }
    obj.extend(textos_si_pedidos(paths, args, h)?);
    Ok(Value::Object(obj))
}

/// The text PVsyst draws in `h` (OCR), only when `ocr:true` was asked for.
fn textos_si_pedidos(
    paths: &crate::config::Paths,
    args: &Value,
    h: Hwnd,
) -> Result<serde_json::Map<String, Value>> {
    if !flag(args, &["ocr"]) {
        return Ok(serde_json::Map::new());
    }
    let ficha = super::ocr::formulario(paths, &json!({"ventana": format!("0x{h:X}")}))?;
    let textos = ficha.get("textos").cloned().unwrap_or(Value::Null);
    Ok(serde_json::Map::from_iter([("textos".to_string(), textos)]))
}

#[cfg(test)]
#[path = "../../../tests/unit/adapters/gui/captura.rs"]
mod tests;
