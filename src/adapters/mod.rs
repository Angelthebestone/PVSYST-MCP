//! Outbound adapters to the PVsyst executables: the documented CLI and the
//! Win32 desktop GUI.

#[expect(
    clippy::excessive_nesting,
    clippy::indexing_slicing,
    clippy::too_many_arguments,
    reason = "deuda medida; se limpia en el paso 2 de docs/handoff_limpieza_optimizacion.md"
)]
pub(crate) mod cli;
pub(crate) mod gui;
pub(crate) mod http;
#[expect(
    clippy::indexing_slicing,
    clippy::too_many_arguments,
    reason = "deuda medida; se limpia en el paso 2 de docs/handoff_limpieza_optimizacion.md"
)]
pub(crate) mod ledger;
#[cfg(windows)]
#[expect(
    clippy::excessive_nesting,
    clippy::indexing_slicing,
    clippy::too_many_arguments,
    reason = "deuda medida; se limpia en el paso 2 de docs/handoff_limpieza_optimizacion.md"
)]
pub(crate) mod uia;
