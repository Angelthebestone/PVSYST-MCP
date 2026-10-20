//! Cooperative cancellation and progress reporting for long-running tool calls.
//!
//! Deliberately neutral: neither the server nor the CLI adapters are referenced
//! here, so any layer can accept a [`SeguimientoTarea`] without a dependency
//! cycle. A detached task uses [`SeguimientoTarea::inerte`].

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

/// Callback invoked with a percentage and a short message.
pub(crate) type Informe<'a> = Box<dyn Fn(f64, &str) + Send + Sync + 'a>;

/// Cancellation flag plus an optional progress sink shared with a running task.
pub(crate) struct SeguimientoTarea<'a> {
    cancelado: Arc<AtomicBool>,
    informar: Option<Informe<'a>>,
}

impl<'a> SeguimientoTarea<'a> {
    /// A tracker that is never cancelled and reports nothing.
    pub(crate) fn inerte() -> SeguimientoTarea<'static> {
        SeguimientoTarea {
            cancelado: Arc::new(AtomicBool::new(false)),
            informar: None,
        }
    }

    /// A tracker bound to `cancelado`, optionally reporting progress.
    pub(crate) fn nuevo(cancelado: Arc<AtomicBool>, informar: Option<Informe<'a>>) -> Self {
        SeguimientoTarea {
            cancelado,
            informar,
        }
    }

    /// Whether the client asked to cancel this task.
    pub(crate) fn esta_cancelado(&self) -> bool {
        self.cancelado.load(Ordering::Relaxed)
    }

    /// Reports progress if a sink was supplied; otherwise does nothing.
    pub(crate) fn informar_progreso(&self, porcentaje: f64, mensaje: &str) {
        if let Some(informar) = &self.informar {
            informar(porcentaje, mensaje);
        }
    }
}

#[cfg(test)]
#[path = "../../tests/unit/progreso/mod.rs"]
mod tests;
