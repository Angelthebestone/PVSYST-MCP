//! GDI resource management and low-level pixel extraction.

use super::ffi::{self, BitmapInfo, BitmapInfoHeader, Hwnd, PW_CLIENTONLY, PW_RENDERFULLCONTENT};
use crate::error::{Error, Result};

const BITS_PER_PIXEL_32: u16 = 32;
const BYTES_PER_PIXEL_32: usize = 4;

struct RecursosGdi {
    win_dc: isize,
    mem_dc: isize,
    bmp: isize,
    old: isize,
}

fn crear_recursos(h: Hwnd, (w, hgt): (i32, i32)) -> Result<RecursosGdi> {
    if !ffi::es_ventana(h) {
        return Err(Error::Gui(format!("Window handle 0x{h:X} is not valid")));
    }
    // SAFETY: Handles comprobados tras cada llamada; toda salida de error libera lo ya creado.
    unsafe {
        let win_dc = ffi::GetDC(h);
        if win_dc == 0 {
            return Err(Error::Gui("GetDC failed".into()));
        }
        let mem_dc = ffi::CreateCompatibleDC(win_dc);
        if mem_dc == 0 {
            ffi::ReleaseDC(h, win_dc);
            return Err(Error::Gui("CreateCompatibleDC failed".into()));
        }
        let bmp = ffi::CreateCompatibleBitmap(win_dc, w, hgt);
        if bmp == 0 {
            ffi::DeleteDC(mem_dc);
            ffi::ReleaseDC(h, win_dc);
            return Err(Error::Gui("CreateCompatibleBitmap failed".into()));
        }
        let old = ffi::SelectObject(mem_dc, bmp);
        if old == 0 || old == -1 {
            ffi::DeleteObject(bmp);
            ffi::DeleteDC(mem_dc);
            ffi::ReleaseDC(h, win_dc);
            return Err(Error::Gui("SelectObject failed".into()));
        }
        Ok(RecursosGdi {
            win_dc,
            mem_dc,
            bmp,
            old,
        })
    }
}

fn liberar_recursos(h: Hwnd, r: &RecursosGdi) {
    // SAFETY: Handles vivos creados por `crear_recursos`; se liberan una sola vez y en orden inverso.
    unsafe {
        ffi::SelectObject(r.mem_dc, r.old);
        ffi::DeleteObject(r.bmp);
        ffi::DeleteDC(r.mem_dc);
        ffi::ReleaseDC(h, r.win_dc);
    }
}

fn renderizar(h: Hwnd, dc: isize) {
    // SAFETY: `dc` es un DC en memoria válido asociado a `h`.
    unsafe {
        if ffi::PrintWindow(h, dc, PW_RENDERFULLCONTENT) == 0 {
            ffi::PrintWindow(h, dc, PW_CLIENTONLY);
        }
    }
}

fn leer_pixeles(mem_dc: isize, bmp: isize, (w, hgt): (i32, i32)) -> Result<Vec<u8>> {
    let mut pixels = vec![0u8; (w * hgt) as usize * BYTES_PER_PIXEL_32];
    let mut bi = BitmapInfo {
        header: BitmapInfoHeader {
            bi_size: std::mem::size_of::<BitmapInfoHeader>() as u32,
            bi_width: w,
            // Negative height: rows are stored top-down.
            bi_height: -hgt,
            bi_planes: 1,
            bi_bit_count: BITS_PER_PIXEL_32,
            bi_compression: 0,
            bi_size_image: 0,
            bi_x_pels: 0,
            bi_y_pels: 0,
            bi_clr_used: 0,
            bi_clr_important: 0,
        },
        colors: [0],
    };
    // SAFETY: pixels tiene w*hgt*4 bytes y bi describe 32 bpp top-down, así que GetDIBits no escribe fuera de rango.
    let lines =
        unsafe { ffi::GetDIBits(mem_dc, bmp, 0, hgt as u32, pixels.as_mut_ptr(), &mut bi, 0) };
    if lines == 0 {
        return Err(Error::Gui("GetDIBits failed".into()));
    }
    Ok(pixels)
}

/// Renders the window `h` into top-down 32-bpp BGRX pixels (`w * hgt * 4` bytes).
pub(crate) fn capture_pixels(h: Hwnd, dims: (i32, i32)) -> Result<Vec<u8>> {
    let recursos = crear_recursos(h, dims)?;
    renderizar(h, recursos.mem_dc);
    let pixels = leer_pixeles(recursos.mem_dc, recursos.bmp, dims);
    liberar_recursos(h, &recursos);
    pixels
}
