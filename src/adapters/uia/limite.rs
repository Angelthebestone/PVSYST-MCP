//! A UIA pattern call (`Invoke`, `Toggle`, `SetValue`…) is synchronous and has no
//! timeout: when PVsyst answers it by opening a modal form, the handler does not
//! return until the form closes. The server runs tools one at a time, so that
//! call would block every later one. The operation runs on its own thread (UIA
//! already owns one COM apartment per call) and the caller stops waiting.

use crate::error::{Error, Result};
use crate::idioma::{Idioma, Texto};
use std::sync::mpsc::{self, RecvTimeoutError};
use std::time::Duration;

/// Runs `tarea` on its own thread and waits at most `espera` for its result.
/// `None` when it did not finish in time: the thread keeps running until the
/// call returns by itself and its result is dropped.
pub(crate) fn con_limite<T, F>(espera: Duration, tarea: F) -> Result<Option<T>>
where
    T: Send + 'static,
    F: FnOnce() -> T + Send + 'static,
{
    let (emisor, receptor) = mpsc::channel();
    std::thread::Builder::new()
        .name("uia-operacion".into())
        .spawn(move || {
            // The receiver is gone once the caller gave up: nobody wants the result.
            let _ = emisor.send(tarea());
        })
        .map_err(|e| {
            // Its only caller (`uia::invocar`) has no `Idioma` to hand down.
            let sin_hilo = Texto {
                es: "No se pudo crear el hilo UIA: {error}",
                en: "Could not create the UIA thread: {error}",
            };
            Error::Gui(sin_hilo.texto_con(Idioma::de_entorno(), &[("error", &e.to_string())]))
        })?;
    match receptor.recv_timeout(espera) {
        Ok(resultado) => Ok(Some(resultado)),
        Err(RecvTimeoutError::Timeout) => Ok(None),
        Err(RecvTimeoutError::Disconnected) => {
            let hilo_caido = Texto {
                es: "La operación UIA terminó de forma inesperada (el hilo falló)",
                en: "The UIA operation ended unexpectedly (the thread failed)",
            };
            Err(Error::Gui(
                hilo_caido.en_idioma(Idioma::de_entorno()).into(),
            ))
        }
    }
}

#[cfg(test)]
#[path = "../../../tests/unit/adapters/uia/limite.rs"]
mod tests;
