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

/// Opciones cuyo valor es un secreto: no se devuelven nunca en la respuesta.
const OPCIONES_SECRETAS: [&str; 4] = ["-k", "--key", "-c", "--customer-id"];

/// Genera una cadena legible representando la línea de comando ejecutada. Es la
/// única salida de un `dry_run`, así que debe poder copiarse tal cual: el
/// ejecutable y los argumentos se citan y se escapan, y los secretos se tachan.
pub(crate) fn display_command_line(exe: &str, args: &[String]) -> String {
    let mut s = citar(exe);
    for a in args {
        s.push(' ');
        s.push_str(&citar(&redactar(a)));
    }
    s
}

/// Tacha el valor de una opción secreta (`-k:<clave>`, `-c:<id>`).
fn redactar(arg: &str) -> String {
    let Some((nombre, _)) = arg.split_once(':') else {
        return arg.to_string();
    };
    if OPCIONES_SECRETAS
        .iter()
        .any(|s| nombre.eq_ignore_ascii_case(s))
    {
        return format!("{nombre}:<REDACTADO>");
    }
    arg.to_string()
}

/// Cita y escapa un argumento con las reglas de `CommandLineToArgvW`: una comilla
/// interior lleva delante un número impar de barras y las barras finales se
/// duplican para que no escapen la comilla de cierre.
fn citar(arg: &str) -> String {
    if !arg.is_empty() && !arg.contains([' ', '\t', '"']) && !arg.ends_with('\\') {
        return arg.to_string();
    }
    let mut s = String::from("\"");
    let mut barras = 0_usize;
    for c in arg.chars() {
        match c {
            '\\' => {
                barras += 1;
                s.push('\\');
            }
            '"' => {
                for _ in 0..=barras {
                    s.push('\\');
                }
                s.push('"');
                barras = 0;
            }
            otro => {
                barras = 0;
                s.push(otro);
            }
        }
    }
    for _ in 0..barras {
        s.push('\\');
    }
    s.push('"');
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

/// Tiempo de espera de `comando`: `run-simulation` usa el de simulación y el resto
/// el corto. Un valor fuera de [1, 3600] s es un error, no un acotado silencioso.
pub(crate) fn timeout_de_comando(args: &Value, comando: &str) -> Result<Duration> {
    let defecto_s = if comando.eq_ignore_ascii_case("run-simulation") {
        TIMEOUT_SIMULACION_S
    } else {
        TIMEOUT_DEFECTO_S
    };
    let Some(valor) = args.get("timeout_s") else {
        return Ok(Duration::from_secs(defecto_s));
    };
    let segundos = valor.as_i64().ok_or_else(|| {
        Error::Invalid(format!(
            "'timeout_s' debe ser un entero entre {TIMEOUT_MINIMO_S} y {TIMEOUT_MAXIMO_S} (recibido {valor})"
        ))
    })?;
    if !(i64::try_from(TIMEOUT_MINIMO_S).unwrap_or(i64::MAX)
        ..=i64::try_from(TIMEOUT_MAXIMO_S).unwrap_or(i64::MAX))
        .contains(&segundos)
    {
        return Err(Error::Invalid(format!(
            "'timeout_s' debe estar entre {TIMEOUT_MINIMO_S} y {TIMEOUT_MAXIMO_S} segundos (recibido {segundos})"
        )));
    }
    Ok(Duration::from_secs(
        u64::try_from(segundos).unwrap_or(defecto_s),
    ))
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

/// Range of PVsystCLI's `-ll` log level.
const LOG_LEVEL_MIN: i64 = 0;
const LOG_LEVEL_MAX: i64 = 3;

/// Rejects a `log_level` outside PVsystCLI's `-ll` range (0 to 3).
pub(crate) fn validar_log_level(args: &Value) -> Result<()> {
    let Some(v) = args.get("log_level").and_then(Value::as_i64) else {
        return Ok(());
    };
    if (LOG_LEVEL_MIN..=LOG_LEVEL_MAX).contains(&v) {
        return Ok(());
    }
    Err(Error::Invalid(format!(
        "'log_level' debe estar entre {LOG_LEVEL_MIN} y {LOG_LEVEL_MAX} (recibido {v})"
    )))
}

#[cfg(test)]
#[path = "../../../tests/unit/adapters/cli/linea_comando.rs"]
mod tests;
