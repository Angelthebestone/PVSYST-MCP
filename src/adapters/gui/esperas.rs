//! Window show synchronization using WinEvent hooks and message pumping.

use super::ffi::{self, enum_windows, Hwnd, EVENT_OBJECT_SHOW, OBJID_WINDOW};
use super::ventanas::{class_of, pid_of, target_pids, text_of, visible, window_json};
use crate::error::{Error, Result};
use serde_json::Value;
use std::cell::{Cell, RefCell};

const MS_POR_SEGUNDO: u64 = 1_000;
const DEFAULT_TIMEOUT_MS: u64 = 10_000;
/// A slow load can finish just after the deadline: one short retry tells
/// "never appeared" from "appeared late" without a second full wait.
const ESPERA_REINTENTO_MS: u64 = 1_500;

/// Window-matching criteria for `esperar`, checked from the hook callback.
#[derive(Clone)]
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

// Decisión de diseño (§4b.5): `thread_local!` es necesario aquí porque el callback Win32 `al_mostrar`
// tiene la signatura fija `WINEVENTPROC` impuesta por `SetWinEventHook`, la cual no admite puntero de
// contexto de usuario. El estado de espera debe residir en variables accesibles desde la función extern "system",
// y `thread_local!` aísla las esperas por hilo impidiendo condiciones de carrera entre hilos concurrentes.
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
        ffi::salir_bucle();
    }
}

fn esperar_evento(criterio: Criterio, timeout_ms: u64) -> Option<Hwnd> {
    OBJETIVO.with(|o| *o.borrow_mut() = Some(criterio));
    HALLADO.with(|x| x.set(0));
    let gancho = ffi::instalar_gancho_mostrar(al_mostrar);
    if gancho != 0 {
        // A window can appear between the caller's scan and the hook: rescan
        // once the hook is live so that event is never lost.
        let ya_visible = OBJETIVO.with(|o| {
            enum_windows()
                .into_iter()
                .find(|h| o.borrow().as_ref().is_some_and(|c| c.coincide(*h)))
        });
        if let Some(h) = ya_visible {
            HALLADO.with(|x| x.set(h));
        } else {
            let ms = timeout_ms.min(u64::from(u32::MAX)) as u32;
            ffi::iniciar_temporizador(ms);
            ffi::bombear_mensajes();
        }
        ffi::retirar_gancho(gancho);
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

/// What a wait found, so "appeared late" is not reported as a plain timeout.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Hallazgo<T> {
    ATiempo(T),
    Tarde(T),
    Nada,
}

/// The retry policy: the deadline first, one immediate re-read, then one short
/// extra wait. A window that shows up in any of them is not a failure.
fn clasificar_espera<T>(
    hallado: Option<T>,
    relectura: impl FnOnce() -> Option<T>,
    reintento: impl FnOnce() -> Option<T>,
) -> Hallazgo<T> {
    if let Some(h) = hallado {
        return Hallazgo::ATiempo(h);
    }
    if let Some(h) = relectura() {
        return Hallazgo::Tarde(h);
    }
    match reintento() {
        Some(h) => Hallazgo::Tarde(h),
        None => Hallazgo::Nada,
    }
}

/// The wait that produced the window, for the answer.
struct Espera<'a> {
    tarde: bool,
    timeout_ms: u64,
    etiqueta: &'a str,
}

/// Adds the wait verdict to the window, so a late arrival is visible in the answer.
fn marcar_espera(window: &mut Value, espera: &Espera<'_>) {
    let Some(mapa) = window.as_object_mut() else {
        return;
    };
    mapa.insert("tarde".into(), serde_json::json!(espera.tarde));
    if espera.tarde {
        mapa.insert("esperado_ms".into(), serde_json::json!(espera.timeout_ms));
        mapa.insert(
            "aviso".into(),
            serde_json::json!(format!(
                "{} appeared after the {} ms deadline: use a longer esperar_segundos next time",
                espera.etiqueta, espera.timeout_ms
            )),
        );
    }
}

fn respuesta_de(h: Hwnd, espera: &Espera<'_>) -> Value {
    let mut window = window_json(h);
    marcar_espera(&mut window, espera);
    window
}

pub(crate) fn esperar_ventana(args: &Value, params: &EsperarParams) -> Result<Value> {
    if params.clase.is_none() && params.titulo.is_none() {
        return Err(Error::Gui(
            "Pass 'clase' or 'titulo' to wait for a window".into(),
        ));
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
    let resultado = clasificar_espera(
        esperar_evento(criterio.clone(), params.timeout_ms),
        || enum_windows().into_iter().find(|h| criterio.coincide(*h)),
        || esperar_evento(criterio.clone(), ESPERA_REINTENTO_MS),
    );
    match resultado {
        Hallazgo::ATiempo(h) => Ok(respuesta_de(
            h,
            &Espera {
                tarde: false,
                timeout_ms: params.timeout_ms,
                etiqueta: &etiqueta,
            },
        )),
        Hallazgo::Tarde(h) => Ok(respuesta_de(
            h,
            &Espera {
                tarde: true,
                timeout_ms: params.timeout_ms,
                etiqueta: &etiqueta,
            },
        )),
        Hallazgo::Nada => Err(Error::Gui(format!(
            "timeout waiting for {etiqueta}: it did not appear in {} ms nor in the {ESPERA_REINTENTO_MS} ms recheck",
            params.timeout_ms
        ))),
    }
}

/// `esperar_segundos` wins over `timeout_ms`; neither gives the default timeout.
fn duracion_ms(args: &Value) -> u64 {
    match args.get("esperar_segundos").and_then(Value::as_u64) {
        Some(segundos) => segundos.saturating_mul(MS_POR_SEGUNDO),
        None => args
            .get("timeout_ms")
            .and_then(Value::as_u64)
            .unwrap_or(DEFAULT_TIMEOUT_MS),
    }
}

pub(crate) fn esperar(args: &Value) -> Result<Value> {
    let timeout = duracion_ms(args);
    let params = EsperarParams {
        clase: args.get("clase").and_then(Value::as_str),
        titulo: args.get("titulo").and_then(Value::as_str),
        timeout_ms: timeout,
    };
    esperar_ventana(args, &params)
}

#[cfg(test)]
#[path = "../../../tests/unit/adapters/gui/esperas.rs"]
mod tests;
