//! GDI extern declarations and bitmap structures.

#[repr(C)]
pub(crate) struct BitmapInfoHeader {
    pub(crate) bi_size: u32,
    pub(crate) bi_width: i32,
    pub(crate) bi_height: i32,
    pub(crate) bi_planes: u16,
    pub(crate) bi_bit_count: u16,
    pub(crate) bi_compression: u32,
    pub(crate) bi_size_image: u32,
    pub(crate) bi_x_pels: i32,
    pub(crate) bi_y_pels: i32,
    pub(crate) bi_clr_used: u32,
    pub(crate) bi_clr_important: u32,
}

#[repr(C)]
pub(crate) struct BitmapInfo {
    pub(crate) header: BitmapInfoHeader,
    pub(crate) colors: [u32; 1],
}

pub(crate) const PW_RENDERFULLCONTENT: u32 = 2;
pub(crate) const PW_CLIENTONLY: u32 = 0;

#[link(name = "user32")]
extern "system" {
    pub(crate) fn GetDC(h: isize) -> isize;
    pub(crate) fn ReleaseDC(h: isize, dc: isize) -> i32;
    pub(crate) fn PrintWindow(h: isize, dc: isize, flags: u32) -> i32;
}

#[link(name = "gdi32")]
extern "system" {
    pub(crate) fn CreateCompatibleDC(dc: isize) -> isize;
    pub(crate) fn CreateCompatibleBitmap(dc: isize, w: i32, h: i32) -> isize;
    pub(crate) fn SelectObject(dc: isize, obj: isize) -> isize;
    pub(crate) fn GetDIBits(
        dc: isize,
        bmp: isize,
        start: u32,
        lines: u32,
        bits: *mut u8,
        bi: *mut BitmapInfo,
        usage: u32,
    ) -> i32;
    pub(crate) fn DeleteObject(obj: isize) -> i32;
    pub(crate) fn DeleteDC(dc: isize) -> i32;
}
