//! Win32 extern declarations and safe wrappers, split by subsystem.
//!
//! Callers use the safe helpers re-exported here; the raw extern blocks live in
//! `ventanas`, `gdi` and `entrada`.

mod entrada;
mod gdi;
mod ventanas;

pub(crate) use entrada::*;
pub(crate) use gdi::*;
pub(crate) use ventanas::*;

pub(crate) type Hwnd = isize;
pub(crate) type EnumProc = Option<unsafe extern "system" fn(Hwnd, isize) -> i32>;
pub(crate) type WinEventProc = unsafe extern "system" fn(isize, u32, Hwnd, i32, i32, u32, u32);
