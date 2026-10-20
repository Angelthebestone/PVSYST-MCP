//! Local configuration and diagnostics without PVsyst or the CLI engine:
//!
//! * `preferencias` / `preferencias_editar` read and edit the GUI `PVsyst.ini`.
//! * `logs_cli` summarises the PVsystCLI admin logs and session file.

pub(crate) mod diagnostico;
pub(crate) mod ini;

pub(crate) use diagnostico::logs_cli;
pub(crate) use ini::{preferencias, preferencias_editar};
