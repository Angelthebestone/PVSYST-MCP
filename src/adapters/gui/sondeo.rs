//! Polling with a deadline: PVsyst exposes no events for "form opened" or
//! "process exited", so the GUI adapters ask again until a time limit.

use std::time::{Duration, Instant};

/// Calls `sondeo` every `intervalo` until it yields a value or `plazo` runs out.
pub(crate) fn sondear_hasta<T, F>(plazo: Duration, intervalo: Duration, mut sondeo: F) -> Option<T>
where
    F: FnMut() -> Option<T>,
{
    let fin = Instant::now() + plazo;
    loop {
        if let Some(valor) = sondeo() {
            return Some(valor);
        }
        if Instant::now() >= fin {
            return None;
        }
        std::thread::sleep(intervalo);
    }
}

/// Waits while `condicion` holds; `false` if it still holds after `plazo`.
pub(crate) fn esperar_mientras<F>(plazo: Duration, intervalo: Duration, mut condicion: F) -> bool
where
    F: FnMut() -> bool,
{
    sondear_hasta(plazo, intervalo, || (!condicion()).then_some(())).is_some()
}
