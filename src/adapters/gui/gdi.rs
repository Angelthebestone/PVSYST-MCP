//! GDI resource management using RAII guards and low-level pixel extraction.

use super::ffi::{self, BitmapInfo, BitmapInfoHeader, Hwnd, PW_CLIENTONLY, PW_RENDERFULLCONTENT};
use crate::error::{Error, Result};

const BITS_PER_PIXEL_32: u16 = 32;
const BYTES_PER_PIXEL_32: usize = 4;

/// RAII guard releasing a window device context on drop.
pub(crate) struct WindowDcGuard {
    hwnd: Hwnd,
    dc: isize,
}

impl WindowDcGuard {
    pub(crate) fn acquire(hwnd: Hwnd) -> Result<Self> {
        // SAFETY: Handle de ventana validado; GetDC devuelve 0 en caso de fallo.
        let dc = unsafe { ffi::GetDC(hwnd) };
        if dc == 0 {
            return Err(Error::Gui("GetDC failed".into()));
        }
        Ok(Self { hwnd, dc })
    }

    pub(crate) fn handle(&self) -> isize {
        self.dc
    }
}

impl Drop for WindowDcGuard {
    fn drop(&mut self) {
        // SAFETY: self.dc es un handle de DC obtenido mediante GetDC(self.hwnd) y se libera una sola vez.
        unsafe {
            ffi::ReleaseDC(self.hwnd, self.dc);
        }
    }
}

/// RAII guard deleting a compatible memory device context on drop.
pub(crate) struct MemoryDcGuard {
    dc: isize,
}

impl MemoryDcGuard {
    pub(crate) fn create(src_dc: isize) -> Result<Self> {
        // SAFETY: src_dc es un DC de ventana válido y no nulo.
        let dc = unsafe { ffi::CreateCompatibleDC(src_dc) };
        if dc == 0 {
            return Err(Error::Gui("CreateCompatibleDC failed".into()));
        }
        Ok(Self { dc })
    }

    pub(crate) fn handle(&self) -> isize {
        self.dc
    }
}

impl Drop for MemoryDcGuard {
    fn drop(&mut self) {
        // SAFETY: self.dc es un DC en memoria obtenido mediante CreateCompatibleDC y se destruye una sola vez.
        unsafe {
            ffi::DeleteDC(self.dc);
        }
    }
}

/// RAII guard deleting a compatible GDI bitmap on drop.
pub(crate) struct BitmapGuard {
    bmp: isize,
}

impl BitmapGuard {
    pub(crate) fn create(src_dc: isize, width: i32, height: i32) -> Result<Self> {
        // SAFETY: src_dc es un DC válido; width y height son dimensiones positivas.
        let bmp = unsafe { ffi::CreateCompatibleBitmap(src_dc, width, height) };
        if bmp == 0 {
            return Err(Error::Gui("CreateCompatibleBitmap failed".into()));
        }
        Ok(Self { bmp })
    }

    pub(crate) fn handle(&self) -> isize {
        self.bmp
    }
}

impl Drop for BitmapGuard {
    fn drop(&mut self) {
        // SAFETY: self.bmp es un handle de bitmap válido creado con CreateCompatibleBitmap y se destruye una sola vez.
        unsafe {
            ffi::DeleteObject(self.bmp);
        }
    }
}

/// RAII guard restoring the previous GDI object selection in a memory DC on drop.
pub(crate) struct SelectedObjectGuard {
    dc: isize,
    old_obj: isize,
}

impl SelectedObjectGuard {
    pub(crate) fn select(dc: isize, obj: isize) -> Result<Self> {
        // SAFETY: dc y obj son handles GDI válidos; SelectObject devuelve el objeto previo o 0/-1 en error.
        let old_obj = unsafe { ffi::SelectObject(dc, obj) };
        if old_obj == 0 || old_obj == -1 {
            return Err(Error::Gui("SelectObject failed".into()));
        }
        Ok(Self { dc, old_obj })
    }
}

impl Drop for SelectedObjectGuard {
    fn drop(&mut self) {
        // SAFETY: Restaura el objeto original en el DC antes de que el nuevo objeto o el DC sean eliminados.
        unsafe {
            ffi::SelectObject(self.dc, self.old_obj);
        }
    }
}

fn leer_dibits(mem_dc: isize, bmp: isize, dims: (i32, i32)) -> Result<Vec<u8>> {
    let (w, hgt) = dims;
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
    let (w, hgt) = dims;
    let win_dc = WindowDcGuard::acquire(h)?;
    let mem_dc = MemoryDcGuard::create(win_dc.handle())?;
    let bitmap = BitmapGuard::create(win_dc.handle(), w, hgt)?;
    let _sel = SelectedObjectGuard::select(mem_dc.handle(), bitmap.handle())?;

    // SAFETY: mem_dc es un DC en memoria válido; PrintWindow dibuja la ventana h en él.
    let ok = unsafe { ffi::PrintWindow(h, mem_dc.handle(), PW_RENDERFULLCONTENT) };
    if ok == 0 {
        // SAFETY: Fallback a copia estándar de cliente si falla la renderización completa.
        unsafe { ffi::PrintWindow(h, mem_dc.handle(), PW_CLIENTONLY) };
    }

    leer_dibits(mem_dc.handle(), bitmap.handle(), (w, hgt))
}
