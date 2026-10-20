//! Registry of cancellation flags for the tool calls currently in flight.

use std::collections::HashMap;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};

/// Active calls keyed by their JSON-RPC request id.
#[derive(Default)]
pub(crate) struct Cancelaciones {
    activas: Mutex<HashMap<String, Arc<AtomicBool>>>,
}

impl Cancelaciones {
    /// Registers a fresh flag for `clave` and returns it.
    pub(crate) fn registrar(&self, clave: String) -> Arc<AtomicBool> {
        let bandera = Arc::new(AtomicBool::new(false));
        if let Ok(mut activas) = self.activas.lock() {
            activas.insert(clave, Arc::clone(&bandera));
        }
        bandera
    }

    /// Marks the call identified by `clave`, if it is still running.
    pub(crate) fn cancelar(&self, clave: &str) {
        if let Ok(activas) = self.activas.lock() {
            if let Some(bandera) = activas.get(clave) {
                bandera.store(true, Ordering::Relaxed);
            }
        }
    }

    /// Forgets a finished call.
    pub(crate) fn quitar(&self, clave: &str) {
        if let Ok(mut activas) = self.activas.lock() {
            activas.remove(clave);
        }
    }
}
