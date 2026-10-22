//! Window enumeration, identification and inspection via Win32.

use super::ffi::{
    self, enum_children, enum_windows, Hwnd, ProcessEntry32W, Rect, TH32CS_SNAPPROCESS,
};
use crate::args::{flag, req_str};
use crate::error::{Error, Result};
use serde_json::{json, Value};

const TEXT_BUFFER_LEN: usize = 512;
const CLASS_BUFFER_LEN: usize = 256;

pub(crate) fn text_of(h: Hwnd) -> String {
    if !ffi::es_ventana(h) {
        return String::new();
    }
    let mut buf = [0u16; TEXT_BUFFER_LEN];
    // SAFETY: `buf` es un array local y se pasa su longitud real; la API escribe como máximo `buf.len()` unidades.
    let n = unsafe { ffi::GetWindowTextW(h, buf.as_mut_ptr(), buf.len() as i32) };
    let slice = buf.get(..n.max(0) as usize).unwrap_or(&[]);
    String::from_utf16_lossy(slice)
}

pub(crate) fn class_of(h: Hwnd) -> String {
    if !ffi::es_ventana(h) {
        return String::new();
    }
    let mut buf = [0u16; CLASS_BUFFER_LEN];
    // SAFETY: `buf` es un array local y se pasa su longitud real; la API escribe como máximo `buf.len()` unidades.
    let n = unsafe { ffi::GetClassNameW(h, buf.as_mut_ptr(), buf.len() as i32) };
    let slice = buf.get(..n.max(0) as usize).unwrap_or(&[]);
    String::from_utf16_lossy(slice)
}

pub(crate) fn pid_of(h: Hwnd) -> u32 {
    let mut pid = 0u32;
    // SAFETY: `pid` es una local viva; la API solo escribe un u32 en ella.
    unsafe {
        ffi::GetWindowThreadProcessId(h, &mut pid);
    }
    pid
}

pub(crate) fn rect_of(h: Hwnd) -> Rect {
    let mut r = Rect::default();
    // SAFETY: `r` es un `Rect` local con el layout de RECT; la API solo escribe en él.
    unsafe {
        ffi::GetWindowRect(h, &mut r);
    }
    r
}

pub(crate) fn visible(h: Hwnd) -> bool {
    ffi::estado_ventana(h).visible
}

/// The first visible child of `h` whose caption is exactly `texto`.
pub(crate) fn hijo_visible_con_texto(h: Hwnd, texto: &str) -> Option<Hwnd> {
    enum_children(h)
        .into_iter()
        .find(|c| visible(*c) && text_of(*c) == texto)
}

/// Top-level windows of process `pid`, in Z-order (front-most first).
pub(crate) fn enum_windows_de(pid: u32) -> Vec<Hwnd> {
    enum_windows()
        .into_iter()
        .filter(|h| pid_of(*h) == pid)
        .collect()
}

pub(crate) fn find_pids(exe_name: &str) -> Vec<u32> {
    let mut pids = Vec::new();
    // SAFETY: `pe` se inicializa a cero con `dw_size` correcto antes de `Process32*`, y el handle `snap` se cierra al terminar.
    unsafe {
        let snap = ffi::CreateToolhelp32Snapshot(TH32CS_SNAPPROCESS, 0);
        if snap == -1 {
            return pids;
        }
        let mut pe: ProcessEntry32W = std::mem::zeroed();
        pe.dw_size = std::mem::size_of::<ProcessEntry32W>() as u32;
        if ffi::Process32FirstW(snap, &mut pe) != 0 {
            loop {
                let fin = pe
                    .sz_exe_file
                    .iter()
                    .position(|&c| c == 0)
                    .unwrap_or(pe.sz_exe_file.len());
                let slice = pe.sz_exe_file.get(..fin).unwrap_or(&[]);
                let name = String::from_utf16_lossy(slice);
                if name.eq_ignore_ascii_case(exe_name) {
                    pids.push(pe.th32_process_id);
                }
                if ffi::Process32NextW(snap, &mut pe) == 0 {
                    break;
                }
            }
        }
        ffi::CloseHandle(snap);
    }
    pids
}

pub(crate) fn window_json(h: Hwnd) -> Value {
    let r = rect_of(h);
    let estado = ffi::estado_ventana(h);
    json!({
        "hwnd": format!("0x{:X}", h),
        "class": class_of(h),
        "title": text_of(h),
        "pid": pid_of(h),
        "visible": estado.visible,
        "enabled": estado.habilitada,
        "minimized": estado.minimizada,
        "rect": { "left": r.left, "top": r.top, "right": r.right, "bottom": r.bottom,
                  "width": r.right - r.left, "height": r.bottom - r.top },
    })
}

pub(crate) fn compact_default(args: &Value) -> bool {
    if let Some(c) = args.get("compacto").and_then(serde_json::Value::as_bool) {
        c
    } else {
        !flag(args, &["completo"])
    }
}

pub(crate) fn window_brief(h: Hwnd) -> Value {
    json!({
        "hwnd": format!("0x{:X}", h),
        "class": class_of(h),
        "title": text_of(h),
        "visible": visible(h),
    })
}

pub(crate) fn target_pids(args: &Value) -> Result<Vec<u32>> {
    if let Some(list) = args.get("pids").and_then(|v| v.as_array()) {
        return Ok(list
            .iter()
            .filter_map(|v| v.as_u64().map(|x| x as u32))
            .collect());
    }
    let name = args
        .get("proceso")
        .and_then(|v| v.as_str())
        .unwrap_or("PVsyst.exe");
    let name = if name.to_lowercase().ends_with(".exe") {
        name.to_string()
    } else {
        format!("{name}.exe")
    };
    let pids = find_pids(&name);
    if pids.is_empty() && args.get("proceso").is_none() {
        return Err(Error::Gui(format!(
            "Process '{name}' is not running. Launch the PVsyst GUI first (or pass 'proceso')."
        )));
    }
    Ok(pids)
}

/// Top-level windows of the target process(es).
pub(crate) fn ventanas_de(args: &Value) -> Result<Vec<Hwnd>> {
    let pids = target_pids(args)?;
    Ok(enum_windows()
        .into_iter()
        .filter(|h| pids.contains(&pid_of(*h)))
        .collect())
}

/// Resolves the window named by the argument `campo` within the target process.
pub(crate) fn hwnd_de(args: &Value, campo: &str) -> Result<Hwnd> {
    let scoped = ventanas_de(args)?;
    let reference = req_str(args, &[campo], "hwnd, class or title")?;
    resolve(&scoped, reference)
        .ok_or_else(|| Error::NotFound(format!("Window '{reference}' not found")))
}

fn matches(win: &Value, needle: &str) -> bool {
    let n = needle.to_lowercase();
    win["title"]
        .as_str()
        .unwrap_or("")
        .to_lowercase()
        .contains(&n)
        || win["class"]
            .as_str()
            .unwrap_or("")
            .to_lowercase()
            .contains(&n)
}

/// Resolve a window reference: `0xHEX`, else class/title substring among the
/// target process' top windows, then among all controls.
pub(crate) fn resolve(haystack: &[Hwnd], reference: &str) -> Option<Hwnd> {
    let hex_val = reference
        .strip_prefix("0x")
        .or_else(|| reference.strip_prefix("0X"))
        .and_then(|hex| isize::from_str_radix(hex, 16).ok());
    if let Some(v) = hex_val {
        if ffi::es_ventana(v) {
            return Some(v);
        }
    }
    let mut fallback: Option<Hwnd> = None;
    for &h in haystack {
        if !matches(&window_json(h), reference) {
            continue;
        }
        if visible(h) {
            return Some(h);
        }
        fallback.get_or_insert(h);
    }
    for c in haystack.iter().flat_map(|&h| enum_children(h)) {
        if !matches(&window_json(c), reference) {
            continue;
        }
        if visible(c) {
            return Some(c);
        }
        fallback.get_or_insert(c);
    }
    fallback
}

/// Lists the top-level windows of the PVsyst process.
pub(crate) fn ventanas(_paths: &crate::config::Paths, args: &Value) -> Result<Value> {
    let pids = target_pids(args)?;
    let compact = compact_default(args);
    let all = enum_windows();
    let mut out = Vec::new();
    for h in all {
        if !flag(args, &["todos"]) && !pids.contains(&pid_of(h)) {
            continue;
        }
        let info = if compact {
            window_brief(h)
        } else {
            window_json(h)
        };
        out.push(info);
    }
    let visible_only = args
        .get("visibles")
        .and_then(Value::as_bool)
        .unwrap_or(!flag(args, &["todos"]));
    if visible_only {
        out.retain(|w| w["visible"].as_bool().unwrap_or(false));
    }
    Ok(json!({
        "processes": pids,
        "count": out.len(),
        "windows": out,
    }))
}

pub(crate) fn controles(_paths: &crate::config::Paths, args: &Value) -> Result<Value> {
    let scoped = ventanas_de(args)?;
    let reference = args
        .get("ventana")
        .and_then(|v| v.as_str())
        .map(std::string::ToString::to_string);
    let roots: Vec<Hwnd> = match &reference {
        Some(r) => {
            vec![resolve(&scoped, r).ok_or_else(|| Error::Gui(format!("Window '{r}' not found")))?]
        }
        None => scoped,
    };
    let compact = compact_default(args);
    let mut out = Vec::new();
    for c in roots.into_iter().flat_map(enum_children) {
        let info = if compact {
            window_brief(c)
        } else {
            window_json(c)
        };
        out.push(info);
    }
    Ok(json!({ "count": out.len(), "controls": out }))
}

pub(crate) fn leer(_paths: &crate::config::Paths, args: &Value) -> Result<Value> {
    let h = hwnd_de(args, "ventana")?;
    Ok(json!({
        "hwnd": format!("0x{:X}", h),
        "class": class_of(h),
        "text": text_of(h),
    }))
}
