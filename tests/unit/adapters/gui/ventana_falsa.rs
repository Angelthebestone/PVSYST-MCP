//! A real top-level window with child windows of chosen classes, texts and
//! visibility, owned by a thread of the test process. It stands in for a PVsyst
//! form: the GUI adapter reads it through the same Win32 calls, hidden children
//! included, and no PVsyst is needed.

use std::sync::mpsc;
use std::thread::JoinHandle;
use windows::core::PCWSTR;
use windows::Win32::Foundation::{HWND, LPARAM, LRESULT, WPARAM};
use windows::Win32::System::Threading::GetCurrentThreadId;
use windows::Win32::UI::Input::KeyboardAndMouse::EnableWindow;
use windows::Win32::UI::WindowsAndMessaging::{
    CreateWindowExW, DefWindowProcW, DestroyWindow, DispatchMessageW, GetMessageW,
    PostThreadMessageW, RegisterClassW, ShowWindow, TranslateMessage, MSG, SW_SHOWNOACTIVATE,
    WINDOW_STYLE, WM_QUIT, WNDCLASSW, WS_CHILD, WS_EX_NOACTIVATE, WS_EX_TOOLWINDOW,
    WS_OVERLAPPEDWINDOW, WS_VISIBLE,
};

/// Where the fake form sits: far outside any monitor, so nothing shows on screen.
const POSICION_FUERA_DE_PANTALLA: i32 = -20_000;
const ANCHO: i32 = 800;
const ALTO: i32 = 600;

/// A child window to create: its class, rectangle in the parent's client area,
/// parent and text.
pub(crate) struct Hijo {
    clase: &'static str,
    texto: &'static str,
    padre: usize,
    rect: (i32, i32, i32, i32),
    visible: bool,
}

impl Hijo {
    /// A visible child of the form (window 0) with no text.
    pub(crate) fn de(clase: &'static str, rect: (i32, i32, i32, i32)) -> Self {
        Self {
            clase,
            texto: "",
            padre: 0,
            rect,
            visible: true,
        }
    }

    /// The same child inside window number `padre` (0 is the form, children count from 1).
    pub(crate) fn en(mut self, padre: usize) -> Self {
        self.padre = padre;
        self
    }

    pub(crate) fn con_texto(mut self, texto: &'static str) -> Self {
        self.texto = texto;
        self
    }

    /// The same child, hidden: a page of a tab strip that is not the current one.
    pub(crate) fn oculto(mut self) -> Self {
        self.visible = false;
        self
    }
}

/// The form and the thread that pumps its messages.
pub(crate) struct VentanaFalsa {
    /// Handle of the form and then of each child, in creation order.
    pub(crate) hwnds: Vec<isize>,
    hilo: u32,
    tarea: Option<JoinHandle<()>>,
}

extern "system" fn procedimiento(h: HWND, msg: u32, w: WPARAM, l: LPARAM) -> LRESULT {
    // SAFETY: reenvía los argumentos recibidos tal cual al procedimiento por defecto de Windows.
    unsafe { DefWindowProcW(h, msg, w, l) }
}

fn ancho_nul(texto: &str) -> Vec<u16> {
    texto.encode_utf16().chain(std::iter::once(0)).collect()
}

fn registrar(clase: &str) {
    let nombre = ancho_nul(clase);
    let ventana = WNDCLASSW {
        lpfnWndProc: Some(procedimiento),
        lpszClassName: PCWSTR(nombre.as_ptr()),
        ..Default::default()
    };
    // SAFETY: `ventana` y `nombre` viven durante la llamada; registrar dos veces la misma clase falla sin efecto.
    unsafe {
        RegisterClassW(&ventana);
    }
}

/// What `CreateWindowExW` needs to build one window.
struct Pedido<'a> {
    clase: &'a str,
    texto: &'a str,
    estilo: u32,
    rect: (i32, i32, i32, i32),
    padre: Option<HWND>,
}

fn crear(pedido: &Pedido<'_>) -> HWND {
    registrar(pedido.clase);
    let (nombre, titulo) = (ancho_nul(pedido.clase), ancho_nul(pedido.texto));
    let (izq, arriba, der, abajo) = pedido.rect;
    let propias = if pedido.padre.is_some() {
        Default::default()
    } else {
        WS_EX_TOOLWINDOW | WS_EX_NOACTIVATE
    };
    // SAFETY: los búferes de texto viven durante la llamada y `padre` es un HWND creado por este hilo o `None`.
    unsafe {
        CreateWindowExW(
            propias,
            PCWSTR(nombre.as_ptr()),
            PCWSTR(titulo.as_ptr()),
            WINDOW_STYLE(pedido.estilo),
            izq,
            arriba,
            der - izq,
            abajo - arriba,
            pedido.padre,
            None,
            None,
            None,
        )
        .unwrap_or_default()
    }
}

/// Creates the form and its children on the current thread; returns their handles.
fn construir(clase: &str, titulo: &str, hijos: &[Hijo]) -> Vec<HWND> {
    let forma = crear(&Pedido {
        clase,
        texto: titulo,
        estilo: WS_OVERLAPPEDWINDOW.0,
        rect: (
            POSICION_FUERA_DE_PANTALLA,
            POSICION_FUERA_DE_PANTALLA,
            POSICION_FUERA_DE_PANTALLA + ANCHO,
            POSICION_FUERA_DE_PANTALLA + ALTO,
        ),
        padre: None,
    });
    let mut hwnds = vec![forma];
    for hijo in hijos {
        let h = crear(&Pedido {
            clase: hijo.clase,
            texto: hijo.texto,
            estilo: WS_CHILD.0 | if hijo.visible { WS_VISIBLE.0 } else { 0 },
            rect: hijo.rect,
            padre: hwnds.get(hijo.padre).copied(),
        });
        hwnds.push(h);
    }
    // SAFETY: `forma` es la ventana que este hilo acaba de crear.
    unsafe {
        let _ = ShowWindow(forma, SW_SHOWNOACTIVATE);
    }
    hwnds
}

impl VentanaFalsa {
    /// A visible form of class `clase` with `hijos`. Class names are registered
    /// on first use and shared by every fake form of the process.
    pub(crate) fn crear(clase: &'static str, titulo: &'static str, hijos: Vec<Hijo>) -> Self {
        let (envio, recibo) = mpsc::channel();
        let tarea = std::thread::spawn(move || {
            let hwnds = construir(clase, titulo, &hijos);
            // SAFETY: sin argumentos; devuelve el identificador del hilo actual.
            let hilo = unsafe { GetCurrentThreadId() };
            let _ = envio.send((hwnds.iter().map(|h| h.0 as isize).collect::<Vec<_>>(), hilo));
            let mut mensaje = MSG::default();
            // SAFETY: `mensaje` es una local viva; el bucle termina con WM_QUIT, que envía `Drop`.
            unsafe {
                while GetMessageW(&mut mensaje, None, 0, 0).as_bool() {
                    let _ = TranslateMessage(&mensaje);
                    DispatchMessageW(&mensaje);
                }
                if let Some(forma) = hwnds.first() {
                    let _ = DestroyWindow(*forma);
                }
            }
        });
        let (hwnds, hilo) = recibo.recv().unwrap_or_default();
        Self {
            hwnds,
            hilo,
            tarea: Some(tarea),
        }
    }

    /// Handle of window `indice` (0 is the form).
    pub(crate) fn hwnd(&self, indice: usize) -> isize {
        self.hwnds.get(indice).copied().unwrap_or_default()
    }

    /// Disables the form, as a VCL modal dialog disables the form below it.
    pub(crate) fn deshabilitar(&self) {
        // SAFETY: `hwnd(0)` es la ventana creada por este objeto, viva hasta su `Drop`.
        unsafe {
            let _ = EnableWindow(HWND(self.hwnd(0) as *mut std::ffi::c_void), false);
        }
    }

    /// The reference `gui_observar` takes for window `indice`.
    pub(crate) fn referencia(&self, indice: usize) -> String {
        format!("0x{:X}", self.hwnd(indice))
    }
}

impl Drop for VentanaFalsa {
    fn drop(&mut self) {
        // SAFETY: publica WM_QUIT en la cola del hilo que creó las ventanas; sin punteros.
        unsafe {
            let _ = PostThreadMessageW(self.hilo, WM_QUIT, WPARAM(0), LPARAM(0));
        }
        if let Some(tarea) = self.tarea.take() {
            let _ = tarea.join();
        }
    }
}
