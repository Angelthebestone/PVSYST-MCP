//! Window show synchronization using WinEvent hooks and message pumping.

use super::ffi::{
    self, enum_windows, Hwnd, Msg, EVENT_OBJECT_SHOW, OBJID_WINDOW, TEMPORIZADOR,
    WINEVENT_OUTOFCONTEXT, WM_TIMER,
};
use super::ventanas::{class_of, pid_of, target_pids, text_of, visible, window_json};
use crate::error::{Error, Result};
use serde_json::Value;
use std::cell::{Cell, RefCell};

const DEFAULT_TIMEOUT_MS: u64 = 10_000;

/// Window-matching criteria for `esperar`, checked from the hook callback.
pub(crate) struct Criterio {
    pub(crate) pids: Vec<u32>,
    pub(crate) clase: Option<String>,
    pub(crate) titulo: Option<String>,
}

impl Criterio {
    pub(crate) fn coincide(&self, h: Hwnd) -> bool {
        if !visible(h) {
            return false;
        }
        if !self.pids.is_empty() && !self.pids.contains(&pid_of(h)) {
            return false;
        }
        let clase_ok = match &self.clase {
            Some(c) => class_of(h).to_lowercase().contains(c.as_str()),
            None => true,
        };
        let titulo_ok = match &self.titulo {
            Some(t) => text_of(h).to_lowercase().contains(t.as_str()),
            None => true,
        };
        clase_ok && titulo_ok
    }
}

thread_local! {
    static OBJETIVO: RefCell<Option<Criterio>> = const { RefCell::new(None) };
    static HALLADO: Cell<Hwnd> = const { Cell::new(0) };
}

unsafe extern "system" fn al_mostrar(
    _gancho: isize,
    evento: u32,
    h: Hwnd,
    id_objeto: i32,
    _hijo: i32,
    _hilo: u32,
    _tiempo: u32,
) {
    if evento != EVENT_OBJECT_SHOW || id_objeto != OBJID_WINDOW || h == 0 {
        return;
    }
    let coincide = OBJETIVO.with(|o| o.borrow().as_ref().is_some_and(|c| c.coincide(h)));
    if coincide {
        HALLADO.with(|x| x.set(h));
        // SAFETY: Envía señal de finalización al bucle de mensajes del hilo.
        unsafe {
            ffi::PostQuitMessage(0);
        }
    }
}

fn bombear_mensajes() {
    loop {
        let mut m = Msg::default();
        // SAFETY: `m` es un `Msg` local con el layout de MSG que la API rellena.
        let r = unsafe { ffi::GetMessageW(&mut m, 0, 0, 0) };
        if r <= 0 || m.message == WM_TIMER {
            break;
        }
        // SAFETY: `m` fue rellenado por `GetMessageW` en esta misma iteración.
        unsafe {
            ffi::TranslateMessage(&m);
            ffi::DispatchMessageW(&m);
        }
    }
}

fn esperar_evento(criterio: Criterio, timeout_ms: u64) -> Option<Hwnd> {
    OBJETIVO.with(|o| *o.borrow_mut() = Some(criterio));
    HALLADO.with(|x| x.set(0));
    // SAFETY: `al_mostrar` tiene la firma WINEVENTPROC y se invoca en este hilo.
    let gancho = unsafe {
        ffi::SetWinEventHook(
            EVENT_OBJECT_SHOW,
            EVENT_OBJECT_SHOW,
            0,
            al_mostrar,
            0,
            0,
            WINEVENT_OUTOFCONTEXT,
        )
    };
    if gancho != 0 {
        let ms = timeout_ms.min(u64::from(u32::MAX)) as u32;
        // SAFETY: Sin ventana ni callback: el temporizador entrega WM_TIMER a la cola de este hilo.
        unsafe { ffi::SetTimer(0, TEMPORIZADOR, ms, 0) };
        bombear_mensajes();
        // SAFETY: `gancho` es el handle devuelto por SetWinEventHook; TEMPORIZADOR es el id usado en SetTimer.
        unsafe {
            ffi::KillTimer(0, TEMPORIZADOR);
            ffi::UnhookWinEvent(gancho);
        }
    }
    OBJETIVO.with(|o| *o.borrow_mut() = None);
    let h = HALLADO.with(Cell::get);
    (h != 0).then_some(h)
}

#[derive(Clone, Copy)]
pub(crate) struct EsperarParams<'a> {
    pub(crate) clase: Option<&'a str>,
    pub(crate) titulo: Option<&'a str>,
    pub(crate) timeout_ms: u64,
}

pub(crate) fn esperar_ventana(args: &Value, params: &EsperarParams) -> Result<Value> {
    if params.clase.is_none() && params.titulo.is_none() {
        return Err(Error::Gui("Indique 'clase' o 'titulo' para esperar".into()));
    }
    let criterio = Criterio {
        pids: target_pids(args)?,
        clase: params.clase.map(str::to_lowercase),
        titulo: params.titulo.map(str::to_lowercase),
    };
    if let Some(h) = enum_windows().into_iter().find(|h| criterio.coincide(*h)) {
        return Ok(window_json(h));
    }
    let etiqueta = params
        .clase
        .or(params.titulo)
        .unwrap_or_default()
        .to_string();
    let h = esperar_evento(criterio, params.timeout_ms)
        .ok_or_else(|| Error::Gui(format!("timeout esperando {etiqueta}")))?;
    Ok(window_json(h))
}

pub(crate) fn esperar(args: &Value) -> Result<Value> {
    let timeout = args
        .get("timeout_ms")
        .and_then(Value::as_u64)
        .unwrap_or(DEFAULT_TIMEOUT_MS);
    let params = EsperarParams {
        clase: args.get("clase").and_then(Value::as_str),
        titulo: args.get("titulo").and_then(Value::as_str),
        timeout_ms: timeout,
    };
    esperar_ventana(args, &params)
}
