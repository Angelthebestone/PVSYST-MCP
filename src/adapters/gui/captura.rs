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

fn aplicar_recorte(handles: (Hwnd, Hwnd), rgb: &[u8], dim: (i32, i32)) -> (Vec<u8>, usize, usize) {
    let (h, raiz) = handles;
    let (w, alto) = dim;
    let rect = rect_of(h);
    let rect_raiz = rect_of(raiz);
    let dims = DimensionesRecorte {
        ancho: w as usize,
        alto: alto as usize,
        x: rect.left - rect_raiz.left,
        y: rect.top - rect_raiz.top,
        ancho_corte: rect.right - rect.left,
        alto_corte: rect.bottom - rect.top,
    };
    recortar_rgb(rgb, &dims)
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
        // SAFETY: Consulta de solo lectura sobre un HWND; un handle inválido devuelve 0.
        unsafe { ffi::GetAncestor(h, GA_ROOT) }
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
    let (rgb_final, ancho, alto_img) = if recorte {
        aplicar_recorte((h, raiz), &rgb, (w, alto))
    } else {
        (rgb, w as usize, alto as usize)
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
mod tests {
    use super::*;
    use std::sync::Arc;

    #[test]
    fn bgrx_is_converted_to_rgb() {
        assert_eq!(rgb_desde_bgrx(&[10, 20, 30, 0]), [30, 20, 10]);
    }

    #[test]
    fn crop_selects_the_requested_rectangle() {
        let rgb: Vec<u8> = (0u8..6).flat_map(|i| [i, i, i]).collect();
        let dims = DimensionesRecorte {
            ancho: 3,
            alto: 2,
            x: 1,
            y: 0,
            ancho_corte: 2,
            alto_corte: 2,
        };
        let (out, cw, ch) = recortar_rgb(&rgb, &dims);
        assert_eq!((cw, ch), (2, 2));
        assert_eq!(out, [1, 1, 1, 2, 2, 2, 4, 4, 4, 5, 5, 5]);
    }

    #[test]
    fn crop_is_clamped_to_the_image() {
        let rgb: Vec<u8> = (0u8..6).flat_map(|i| [i, i, i]).collect();
        let dims = DimensionesRecorte {
            ancho: 3,
            alto: 2,
            x: 5,
            y: -3,
            ancho_corte: 10,
            alto_corte: 10,
        };
        let (out, cw, ch) = recortar_rgb(&rgb, &dims);
        assert_eq!((cw, ch), (1, 2));
        assert_eq!(out, [2, 2, 2, 5, 5, 5]);
    }

    #[test]
    fn should_reject_capture_output_path_outside_workspace_by_default() {
        let ws = std::env::temp_dir().join("test_pvsyst_ws_confined");
        let paths = crate::config::Paths {
            workspace: ws,
            cli_exe: PathBuf::new(),
            gui_exe: PathBuf::new(),
            experto: false,
            estado: Arc::default(),
        };
        let args = json!({
            "ruta": "../../escaped_capture.png"
        });
        let res = ruta_salida(&paths, &args, 0);
        assert!(res.is_err(), "Must reject escaping path by default");
    }

    #[test]
    fn should_allow_capture_output_path_outside_workspace_when_explicitly_permitted() {
        let ws = std::env::temp_dir().join("test_pvsyst_ws_confined");
        let paths = crate::config::Paths {
            workspace: ws,
            cli_exe: PathBuf::new(),
            gui_exe: PathBuf::new(),
            experto: false,
            estado: Arc::default(),
        };
        let outside = std::env::temp_dir().join("safe_capture_outside.png");
        let args = json!({
            "ruta": outside.to_string_lossy(),
            "permitir_fuera_del_workspace": true
        });
        let res = ruta_salida(&paths, &args, 0);
        assert!(
            res.is_ok(),
            "Must allow outside path when explicitly permitted"
        );
    }
}
