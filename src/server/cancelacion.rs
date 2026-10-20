//! Registry of cancellation flags for the tool calls currently in flight.

use std::collections::HashMap;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};

type Banderas = HashMap<String, Arc<AtomicBool>>;

/// Active calls keyed by their JSON-RPC request id.
#[derive(Default)]
pub(crate) struct Cancelaciones {
    activas: Mutex<Banderas>,
}

impl Cancelaciones {
    fn con_activas<T>(&self, f: impl FnOnce(&mut Banderas) -> T) -> Option<T> {
        match self.activas.lock() {
            Ok(mut activas) => Some(f(&mut activas)),
            Err(_) => {
                tracing::warn!("registro de cancelaciones envenenado");
                None
            }
        }
    }

    /// Registers a fresh flag for `clave` and returns it.
    pub(crate) fn registrar(&self, clave: String) -> Arc<AtomicBool> {
        let bandera = Arc::new(AtomicBool::new(false));
        self.con_activas(|activas| {
            activas.insert(clave, Arc::clone(&bandera));
        });
        bandera
    }

    /// Marks the call identified by `clave`, if it is still running.
    pub(crate) fn cancelar(&self, clave: &str) {
        self.con_activas(|activas| {
            if let Some(bandera) = activas.get(clave) {
                bandera.store(true, Ordering::Relaxed);
            }
        });
    }

    /// Forgets a finished call, but only if its registration is still the same one.
    pub(crate) fn quitar(&self, clave: &str, bandera: &Arc<AtomicBool>) {
        self.con_activas(|activas| {
            if activas
                .get(clave)
                .is_some_and(|registrada| Arc::ptr_eq(registrada, bandera))
            {
                activas.remove(clave);
            }
        });
    }
}
