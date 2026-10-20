//! Formateo de argumentos y resolución de ejecutables para `PVsystCLI.exe`.
//!
//! Encapsula la búsqueda del ejecutable en el sistema, la validación de su existencia
//! y el cálculo de límites de tiempo para llamadas a la CLI.

use crate::config::Paths;
use crate::error::{Error, Result};
use serde_json::Value;
use std::path::PathBuf;
use std::time::Duration;

/// Límite de tiempo por defecto para comandos CLI ordinarios (en segundos).
pub(crate) const TIMEOUT_DEFECTO_S: u64 = 120;

/// Límite de tiempo por defecto para `run-simulation` (en segundos).
pub(crate) const TIMEOUT_SIMULACION_S: u64 = 900;

/// Límite inferior permitido para el parámetro de tiempo de espera (1 segundo).
const TIMEOUT_MINIMO_S: u64 = 1;

/// Límite superior permitido para el parámetro de tiempo de espera (1 hora).
const TIMEOUT_MAXIMO_S: u64 = 3600;

/// Genera una cadena legible representando la línea de comando ejecutada.
pub(crate) fn display_command_line(exe: &str, args: &[String]) -> String {
    let mut s = exe.to_string();
    for a in args {
        s.push(' ');
        if a.contains(' ') {
            s.push('"');
            s.push_str(a);
            s.push('"');
        } else {
            s.push_str(a);
        }
    }
    s
}

/// Localiza un comando en el PATH del sistema o verifica su ruta directa.
pub(crate) fn which(cmd: &str) -> Option<PathBuf> {
    if cmd.contains('\\') || cmd.contains('/') {
        return PathBuf::from(cmd).exists().then(|| PathBuf::from(cmd));
    }
    let path = std::env::var_os("PATH")?;
    for dir in std::env::split_paths(&path) {
        let cand = dir.join(cmd);
        if cand.exists() {
            return Some(cand);
        }
        let with_exe = dir.join(format!("{cmd}.exe"));
        if with_exe.exists() {
            return Some(with_exe);
        }
    }
    None
}

/// Comprueba si el ejecutable de PVsystCLI está disponible en disco o PATH.
pub(crate) fn cli_available(paths: &Paths) -> bool {
    paths.cli_exe.exists() || which(&paths.cli_exe.to_string_lossy()).is_some()
}

/// Extrae y normaliza el límite de tiempo solicitado en `args`, acotado a [1, 3600] s.
pub(crate) fn timeout_de(args: &Value, defecto_s: u64) -> Duration {
    let s = args
        .get("timeout_s")
        .and_then(Value::as_u64)
        .unwrap_or(defecto_s)
        .clamp(TIMEOUT_MINIMO_S, TIMEOUT_MAXIMO_S);
    Duration::from_secs(s)
}

/// Exige que PVsystCLI esté instalado y disponible, fallando con [`Error::Cli`] si no.
pub(crate) fn requerir_cli(paths: &Paths) -> Result<()> {
    if cli_available(paths) {
        Ok(())
    } else {
        Err(Error::Cli(format!(
            "PVsystCLI.exe not found at '{}'. Set PVSYST_CLI.",
            paths.cli_exe.display()
        )))
    }
}

#[cfg(test)]
#[path = "../../../tests/unit/adapters/cli/linea_comando.rs"]
mod tests;
