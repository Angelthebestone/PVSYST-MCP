//! Serialized protocol output shared by the main loop and the worker thread.
//!
//! Responses to immediate requests and worker responses may be written from
//! different threads, so one mutex keeps each JSON-RPC line whole.

use std::io::{self, Write};
use std::sync::Mutex;

/// Writer that serializes complete newline-delimited messages.
pub(crate) struct Salida<W> {
    pub(crate) inner: Mutex<W>,
}

impl<W: Write> Salida<W> {
    /// Wraps a writer behind the output mutex.
    pub(crate) fn nueva(inner: W) -> Self {
        Salida {
            inner: Mutex::new(inner),
        }
    }

    /// Writes one message followed by a newline and flushes it.
    pub(crate) fn escribir_linea(&self, linea: &str) -> io::Result<()> {
        let mut salida = self
            .inner
            .lock()
            .map_err(|_| io::Error::other("salida envenenada"))?;
        writeln!(salida, "{linea}")?;
        salida.flush()
    }
}
