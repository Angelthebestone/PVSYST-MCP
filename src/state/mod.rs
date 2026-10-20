//! State shared by the tool calls of one server: background CLI jobs and the UI
//! Automation snapshot cache. It lives inside [`crate::Paths`] (an `Arc`), so every
//! server instance owns its own and nothing is process-global.
//!
//! The record types are plain data defined here (not in the features that fill
//! them) so `state`, which `config::Paths` holds, depends on nothing but `error`
//! and the module graph stays acyclic.

mod registros;

pub(crate) use registros::{CapturaFlujo, CliOutput, Pendiente, Plan, Sink, Trabajo};

use serde_json::Value;
use std::collections::HashMap;
use std::fmt;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Mutex;
use std::time::Instant;

/// Cached UIA tree keyed by `(hwnd, depth, max nodes)`.
pub(crate) type CacheArboles = Mutex<HashMap<(isize, u32, u32), (Instant, Value)>>;

/// Registries that outlive a single tool call.
#[derive(Default)]
pub(crate) struct Estado {
    /// Running or finished `PVsystCLI` jobs (`simular {segundo_plano:true}`).
    pub(crate) trabajos: Mutex<HashMap<String, Trabajo>>,
    /// Jobs waiting for their result to be finalised exactly once.
    pub(crate) pendientes: Mutex<HashMap<String, Pendiente>>,
    /// Recent UIA tree snapshots.
    pub(crate) arboles: CacheArboles,
    secuencia: AtomicU64,
}

impl Estado {
    /// Next background job id, `sim-1`, `sim-2`, ...
    pub(crate) fn siguiente_trabajo(&self) -> String {
        format!("sim-{}", self.secuencia.fetch_add(1, Ordering::SeqCst) + 1)
    }
}

impl fmt::Debug for Estado {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Estado").finish_non_exhaustive()
    }
}

#[cfg(test)]
#[path = "../../tests/unit/state/mod.rs"]
mod tests;
