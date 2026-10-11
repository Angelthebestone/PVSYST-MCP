//! Snapshot history and atomic writes for files the MCP modifies.
//!
//! Tools that mutate existing files can take a snapshot of the old content
//! before they write, so any change can be reverted with [`restaurar`].
//! Snapshots live under `<workspace>/UserData/mcp_history/<id>/` with a
//! `manifest.json` describing the original paths and the copies.

mod escritura_atomica;
mod restauracion;
mod snapshot;

#[cfg(test)]
mod tests;
#[cfg(test)]
mod tests_externos;

pub(crate) use escritura_atomica::{escribir_atomico, reemplazar};
pub(crate) use restauracion::restaurar;
pub(crate) use snapshot::{listar, sello_compacto, snapshot};
