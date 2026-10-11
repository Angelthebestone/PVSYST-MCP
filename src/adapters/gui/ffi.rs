//! Win32 extern declarations and GDI/User32/Kernel32 data structures.

pub(crate) type Hwnd = isize;
pub(crate) type EnumProc = Option<unsafe extern "system" fn(Hwnd, isize) -> i32>;
pub(crate) type WinEventProc = unsafe extern "system" fn(isize, u32, Hwnd, i32, i32, u32, u32);

pub(crate) const WM_CLOSE: u32 = 0x0010;
pub(crate) const WM_KEYDOWN: u32 = 0x0100;
pub(crate) const WM_KEYUP: u32 = 0x0101;
pub(crate) const WM_LBUTTONDOWN: u32 = 0x0201;
pub(crate) const WM_LBUTTONUP: u32 = 0x0202;
pub(crate) const WM_TIMER: u32 = 0x0113;
pub(crate) const BM_CLICK: u32 = 0x00F5;
pub(crate) const SW_RESTORE: i32 = 9;
pub(crate) const EVENT_OBJECT_SHOW: u32 = 0x8002;
pub(crate) const WINEVENT_OUTOFCONTEXT: u32 = 0x0000;
pub(crate) const OBJID_WINDOW: i32 = 0;
pub(crate) const DESKTOP_READOBJECTS: u32 = 0x0001;
pub(crate) const MOUSEEVENTF_LEFTDOWN: u32 = 0x0002;
pub(crate) const MOUSEEVENTF_LEFTUP: u32 = 0x0004;
pub(crate) const GA_ROOT: u32 = 2;
pub(crate) const PW_RENDERFULLCONTENT: u32 = 2;
pub(crate) const PW_CLIENTONLY: u32 = 0;
pub(crate) const TH32CS_SNAPPROCESS: u32 = 0x0000_0002;
pub(crate) const TEMPORIZADOR: usize = 1;

#[repr(C)]
#[derive(Clone, Copy, Default)]
pub(crate) struct Rect {
    pub(crate) left: i32,
    pub(crate) top: i32,
    pub(crate) right: i32,
    pub(crate) bottom: i32,
}

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

#[repr(C)]
#[derive(Default)]
pub(crate) struct Msg {
    pub(crate) hwnd: Hwnd,
    pub(crate) message: u32,
    pub(crate) wparam: usize,
    pub(crate) lparam: isize,
    pub(crate) time: u32,
    pub(crate) pt_x: i32,
    pub(crate) pt_y: i32,
}

#[repr(C)]
pub(crate) struct ProcessEntry32W {
    pub(crate) dw_size: u32,
    pub(crate) cnt_usage: u32,
    pub(crate) th32_process_id: u32,
    pub(crate) th32_default_heap_id: usize,
    pub(crate) th32_module_id: u32,
    pub(crate) cnt_threads: u32,
    pub(crate) th32_parent_process_id: u32,
    pub(crate) pc_pri_class_base: i32,
    pub(crate) dw_flags: u32,
    pub(crate) sz_exe_file: [u16; 260],
}

#[link(name = "user32")]
extern "system" {
    pub(crate) fn EnumWindows(cb: EnumProc, l: isize) -> i32;
    pub(crate) fn EnumChildWindows(parent: Hwnd, cb: EnumProc, l: isize) -> i32;
    pub(crate) fn GetWindowTextW(h: Hwnd, s: *mut u16, n: i32) -> i32;
    pub(crate) fn GetClassNameW(h: Hwnd, s: *mut u16, n: i32) -> i32;
    pub(crate) fn GetWindowThreadProcessId(h: Hwnd, pid: *mut u32) -> u32;
    pub(crate) fn IsWindowVisible(h: Hwnd) -> i32;
    pub(crate) fn IsWindowEnabled(h: Hwnd) -> i32;
    pub(crate) fn IsWindow(h: Hwnd) -> i32;
    pub(crate) fn IsIconic(h: Hwnd) -> i32;
    pub(crate) fn GetWindowRect(h: Hwnd, r: *mut Rect) -> i32;
    pub(crate) fn GetClientRect(h: Hwnd, r: *mut Rect) -> i32;
    pub(crate) fn SendMessageW(h: Hwnd, msg: u32, w: usize, l: isize) -> isize;
    pub(crate) fn PostMessageW(h: Hwnd, msg: u32, w: usize, l: isize) -> i32;
    pub(crate) fn ShowWindow(h: Hwnd, cmd: i32) -> i32;
    pub(crate) fn SetForegroundWindow(h: Hwnd) -> i32;
    pub(crate) fn GetAncestor(h: Hwnd, flags: u32) -> Hwnd;
    pub(crate) fn GetDC(h: Hwnd) -> isize;
    pub(crate) fn ReleaseDC(h: Hwnd, dc: isize) -> i32;
    pub(crate) fn PrintWindow(h: Hwnd, dc: isize, flags: u32) -> i32;
    pub(crate) fn GetForegroundWindow() -> Hwnd;
    pub(crate) fn SetCursorPos(x: i32, y: i32) -> i32;
    pub(crate) fn mouse_event(flags: u32, dx: u32, dy: u32, data: u32, extra: usize);
    pub(crate) fn OpenInputDesktop(flags: u32, inherit: i32, access: u32) -> isize;
    pub(crate) fn CloseDesktop(h: isize) -> i32;
    pub(crate) fn SetWinEventHook(
        min: u32,
        max: u32,
        module: isize,
        cb: WinEventProc,
        pid: u32,
        tid: u32,
        flags: u32,
    ) -> isize;
    pub(crate) fn UnhookWinEvent(h: isize) -> i32;
    pub(crate) fn GetMessageW(msg: *mut Msg, h: Hwnd, min: u32, max: u32) -> i32;
    pub(crate) fn TranslateMessage(msg: *const Msg) -> i32;
    pub(crate) fn DispatchMessageW(msg: *const Msg) -> isize;
    pub(crate) fn SetTimer(h: Hwnd, id: usize, ms: u32, cb: isize) -> usize;
    pub(crate) fn KillTimer(h: Hwnd, id: usize) -> i32;
    pub(crate) fn PostQuitMessage(code: i32);
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

#[link(name = "kernel32")]
extern "system" {
    pub(crate) fn CreateToolhelp32Snapshot(flags: u32, pid: u32) -> isize;
    pub(crate) fn Process32FirstW(snap: isize, pe: *mut ProcessEntry32W) -> i32;
    pub(crate) fn Process32NextW(snap: isize, pe: *mut ProcessEntry32W) -> i32;
    pub(crate) fn CloseHandle(h: isize) -> i32;
}

unsafe extern "system" fn collect_enum(h: Hwnd, l: isize) -> i32 {
    let out = &mut *(l as *mut Vec<Hwnd>);
    out.push(h);
    1
}

pub(crate) fn enum_windows() -> Vec<Hwnd> {
    let mut v: Vec<Hwnd> = Vec::new();
    // SAFETY: `collect_enum` solo reinterpreta `lparam` como el `Vec<Hwnd>` local `v`, vivo y sin alias durante la enumeración síncrona.
    unsafe {
        EnumWindows(Some(collect_enum), &mut v as *mut _ as isize);
    }
    v
}

pub(crate) fn enum_children(parent: Hwnd) -> Vec<Hwnd> {
    let mut v: Vec<Hwnd> = Vec::new();
    // SAFETY: `collect_enum` solo reinterpreta `lparam` como el `Vec<Hwnd>` local `v`, vivo y sin alias durante la enumeración síncrona.
    unsafe {
        EnumChildWindows(parent, Some(collect_enum), &mut v as *mut _ as isize);
    }
    v
}
