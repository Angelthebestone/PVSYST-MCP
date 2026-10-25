//! Overwrite guard for component writes: an existing target needs `sobrescribir: true`.

use crate::args::flag;
use crate::error::{Error, Result};
use serde_json::Value;
use std::path::Path;

/// Reject writing over an existing component unless the caller passed `sobrescribir: true`.
pub(crate) fn exigir_sobrescribir(target: &Path, args: &Value) -> Result<()> {
    if !target.exists() || flag(args, &["sobrescribir"]) {
        return Ok(());
    }
    Err(Error::Invalid(format!(
        "'{}' ya existe: pasa 'sobrescribir': true para reemplazarlo",
        target.display()
    )))
}
