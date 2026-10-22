//! GDI screen capture workflow, image cropping and PNG file writing.

use super::base64::base64;
use super::ffi::{self, Hwnd, GA_ROOT};
use super::gdi::capture_pixels;
use super::png::png_rgb;
use super::ventanas::{class_of, hwnd_de, rect_of, text_of};
use crate::args::flag;
use crate::error::{Error, Result};
use crate::workspace::{resolver_ruta, Alcance};
use serde_json::{json, Value};
use std::path::PathBuf;

const BYTES_PER_PIXEL_32: usize = 4;
const BYTES_PER_PIXEL_RGB: usize = 3;

/// Converts top-down BGRX pixels to packed RGB.
pub(crate) fn rgb_desde_bgrx(px: &[u8]) -> Vec<u8> {
    px.chunks(BYTES_PER_PIXEL_32)
        .filter(|p| p.len() == BYTES_PER_PIXEL_32)
        .flat_map(|p| [p[2], p[1], p[0]])
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
    let x = dims.x.clamp(0, dims.ancho as i32 - 1) as usize;
    let y = dims.y.clamp(0, dims.alto as i32 - 1) as usize;
    let cw = (dims.ancho_corte.max(1) as usize).min(dims.ancho - x);
    let ch = (dims.alto_corte.max(1) as usize).min(dims.alto - y);
    let mut out = Vec::with_capacity(cw * ch * BYTES_PER_PIXEL_RGB);
    for fila in y..y + ch {
        let ini = (fila * dims.ancho + x) * BYTES_PER_PIXEL_RGB;
        out.extend_from_slice(&rgb[ini..ini + cw * BYTES_PER_PIXEL_RGB]);
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
    let png = png_rgb(ancho as u32, alto_img as u32, &rgb_final);
    let out = ruta_salida(paths, args, h)?;
    escribir_png(&out, &png)?;
    let mut res = json!({
        "hwnd": format!("0x{:X}", h),
        "class": class_of(h),
        "title": text_of(h),
        "path": out.to_string_lossy(),
        "width": ancho,
        "height": alto_img,
        "bytes": png.len(),
    });
    if flag(args, &["imagen"]) {
        res["_imagen_png_base64"] = json!(base64(&png));
    }
    Ok(res)
}

#[cfg(test)]
#[path = "../../../tests/unit/adapters/gui/captura.rs"]
mod tests;
