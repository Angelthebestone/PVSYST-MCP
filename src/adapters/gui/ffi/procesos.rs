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

#[link(name = "kernel32")]
extern "system" {
    fn CreateToolhelp32Snapshot(flags: u32, pid: u32) -> isize;
    fn Process32FirstW(snap: isize, pe: *mut ProcessEntry32W) -> i32;
    fn Process32NextW(snap: isize, pe: *mut ProcessEntry32W) -> i32;
    fn CloseHandle(h: isize) -> i32;
}

/// PIDs of the running processes whose executable is `ejecutable`
/// (case-insensitive). Complexity: O(processes).
pub(crate) fn pids_de_ejecutable(ejecutable: &str) -> Vec<u32> {
    // SAFETY: plain integer arguments; the returned handle is checked below.
    let snap = unsafe { CreateToolhelp32Snapshot(TH32CS_SNAPPROCESS, 0) };
    if snap == HANDLE_INVALIDO {
        return Vec::new();
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
    let mut pids = Vec::new();
    // SAFETY: `snap` is a valid snapshot handle and `pe` a live, correctly sized
    // `PROCESSENTRY32W` (`dw_size` set) that the API fills on each step.
    let mut hay_entrada = unsafe { Process32FirstW(snap, &mut pe) } != 0;
    while hay_entrada {
        if pe.nombre_ejecutable().eq_ignore_ascii_case(ejecutable) {
            pids.push(pe.th32_process_id);
        }
        // SAFETY: same handle and entry as above.
        hay_entrada = unsafe { Process32NextW(snap, &mut pe) } != 0;
    }
    // SAFETY: `snap` was opened above and is closed exactly once.
    unsafe { CloseHandle(snap) };
    pids
}
