//! Process enumeration through the Toolhelp snapshot API.

const TH32CS_SNAPPROCESS: u32 = 0x0000_0002;
const HANDLE_INVALIDO: isize = -1;
/// `MAX_PATH`: size of `PROCESSENTRY32W::szExeFile`.
const LARGO_RUTA: usize = 260;

#[repr(C)]
struct ProcessEntry32W {
    dw_size: u32,
    cnt_usage: u32,
    th32_process_id: u32,
    th32_default_heap_id: usize,
    th32_module_id: u32,
    cnt_threads: u32,
    th32_parent_process_id: u32,
    pc_pri_class_base: i32,
    dw_flags: u32,
    sz_exe_file: [u16; LARGO_RUTA],
}

impl ProcessEntry32W {
    fn nombre_ejecutable(&self) -> String {
        let fin = self
            .sz_exe_file
            .iter()
            .position(|&c| c == 0)
            .unwrap_or(LARGO_RUTA);
        String::from_utf16_lossy(self.sz_exe_file.get(..fin).unwrap_or(&[]))
    }
}

/// Access right `TerminateProcess` needs.
const PROCESS_TERMINATE: u32 = 0x0001;

#[link(name = "kernel32")]
extern "system" {
    fn CreateToolhelp32Snapshot(flags: u32, pid: u32) -> isize;
    fn Process32FirstW(snap: isize, pe: *mut ProcessEntry32W) -> i32;
    fn Process32NextW(snap: isize, pe: *mut ProcessEntry32W) -> i32;
    fn OpenProcess(access: u32, heredable: i32, pid: u32) -> isize;
    fn TerminateProcess(handle: isize, code: u32) -> i32;
    fn CloseHandle(h: isize) -> i32;
}

/// Ends process `pid`; `false` only when it is still alive afterwards.
///
/// The process can die between `OpenProcess` and `TerminateProcess` (or between
/// the snapshot and either call): a failed call then means it is already gone,
/// which is the outcome the caller wanted.
pub(crate) fn terminar_proceso(pid: u32) -> bool {
    // SAFETY: plain integer arguments; a null handle is handled below.
    let handle = unsafe { OpenProcess(PROCESS_TERMINATE, 0, pid) };
    if handle == 0 {
        return !proceso_vivo(pid);
    }
    // SAFETY: `handle` was opened above with `PROCESS_TERMINATE` and is closed
    // exactly once below.
    let terminado = unsafe { TerminateProcess(handle, 1) } != 0;
    // SAFETY: same handle, closed once.
    unsafe { CloseHandle(handle) };
    terminado || !proceso_vivo(pid)
}

/// Whether `pid` is in the process snapshot right now. Complexity: O(processes).
/// A snapshot that cannot be taken is reported as alive: the caller must not
/// take a failed query for a dead process.
pub(crate) fn proceso_vivo(pid: u32) -> bool {
    let mut vivo = false;
    let consultado = recorrer_procesos(|pe| {
        vivo = pe.th32_process_id == pid;
        vivo
    });
    !consultado || vivo
}

/// PIDs of the running processes whose executable is `ejecutable`
/// (case-insensitive). Complexity: O(processes).
pub(crate) fn pids_de_ejecutable(ejecutable: &str) -> Vec<u32> {
    let mut pids = Vec::new();
    recorrer_procesos(|pe| {
        if pe.nombre_ejecutable().eq_ignore_ascii_case(ejecutable) {
            pids.push(pe.th32_process_id);
        }
        false
    });
    pids
}

/// Calls `visitar` on each process entry until it returns `true`; `false` when
/// the snapshot could not be taken.
fn recorrer_procesos(mut visitar: impl FnMut(&ProcessEntry32W) -> bool) -> bool {
    // SAFETY: plain integer arguments; the returned handle is checked below.
    let snap = unsafe { CreateToolhelp32Snapshot(TH32CS_SNAPPROCESS, 0) };
    if snap == HANDLE_INVALIDO {
        return false;
    }
    let mut pe = ProcessEntry32W {
        dw_size: std::mem::size_of::<ProcessEntry32W>() as u32,
        cnt_usage: 0,
        th32_process_id: 0,
        th32_default_heap_id: 0,
        th32_module_id: 0,
        cnt_threads: 0,
        th32_parent_process_id: 0,
        pc_pri_class_base: 0,
        dw_flags: 0,
        sz_exe_file: [0; LARGO_RUTA],
    };
    // SAFETY: `snap` is a valid snapshot handle and `pe` a live, correctly sized
    // `PROCESSENTRY32W` (`dw_size` set) that the API fills on each step.
    let mut hay_entrada = unsafe { Process32FirstW(snap, &mut pe) } != 0;
    while hay_entrada {
        if visitar(&pe) {
            break;
        }
        // SAFETY: same handle and entry as above.
        hay_entrada = unsafe { Process32NextW(snap, &mut pe) } != 0;
    }
    // SAFETY: `snap` was opened above and is closed exactly once.
    unsafe { CloseHandle(snap) };
    true
}
