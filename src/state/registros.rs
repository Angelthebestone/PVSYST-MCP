//! Plain records kept in [`super::Estado`]. Behaviour lives with the features
//! that fill them (`adapters::cli`, `simulation`).

use crate::error::Result;
use serde_json::Value;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};

/// Bytes captured from one child stream, capped by `adapters::cli`.
#[derive(Debug, Default)]
pub(crate) struct CapturaFlujo {
    pub(crate) bytes: Vec<u8>,
    pub(crate) truncado: bool,
    pub(crate) error_io: Option<String>,
}

/// Stream capture shared between the reader thread and observers.
pub(crate) type Sink = Arc<Mutex<CapturaFlujo>>;

/// Complete outcome of one PVsystCLI run.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct CliOutput {
    pub exit_code: Option<i32>,
    pub stdout: String,
    pub stderr: String,
    pub command_line: String,
    pub timed_out: bool,
    pub timeout_s: u64,
    pub stdout_truncado: bool,
    pub stderr_truncado: bool,
}

/// A PVsystCLI process launched in the background.
pub(crate) struct Trabajo {
    pub(crate) sink: Sink,
    pub(crate) resultado: Option<Result<CliOutput>>,
}

/// What is needed to close a simulation run once PVsystCLI finishes.
pub(crate) struct Plan {
    pub(crate) workspace: PathBuf,
    pub(crate) clave: Option<String>,
    pub(crate) advertencia_cache: Option<String>,
    pub(crate) ocf: Option<String>,
    pub(crate) rpf: Option<String>,
    pub(crate) antes: (Option<u64>, Option<u64>),
    pub(crate) ejecuciones: u32,
}

/// Background simulation waiting for its result to be finalised (`estado`).
pub(crate) struct Pendiente {
    pub(crate) plan: Plan,
    pub(crate) comando: String,
    pub(crate) contexto: Value,
    pub(crate) resultado: Option<Value>,
}
