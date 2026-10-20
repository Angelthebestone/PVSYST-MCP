//! Validation for user-supplied file and folder stems.

use crate::error::{Error, Result};

/// Windows device names that cannot be used as file or folder stems.
const NOMBRES_RESERVADOS: &[&str] = &[
    "CON", "PRN", "AUX", "NUL", "COM1", "COM2", "COM3", "COM4", "COM5", "COM6", "COM7", "COM8",
    "COM9", "LPT1", "LPT2", "LPT3", "LPT4", "LPT5", "LPT6", "LPT7", "LPT8", "LPT9",
];

/// Validate a user-supplied file or project name before it is joined into a path.
pub(crate) fn validar_nombre(nombre: &str) -> Result<()> {
    let razon = if nombre.is_empty() {
        "vacío"
    } else if nombre.chars().any(|c| "/\\:*?\"<>|".contains(c)) {
        "contiene un carácter no permitido (/ \\ : * ? \" < > |)"
    } else if nombre.contains("..") {
        "contiene '..'"
    } else if nombre.chars().any(char::is_control) {
        "contiene caracteres de control"
    } else if nombre.starts_with([' ', '.']) || nombre.ends_with([' ', '.']) {
        "no puede empezar o terminar con espacio o punto"
    } else if NOMBRES_RESERVADOS
        .iter()
        .any(|r| r.eq_ignore_ascii_case(nombre.split('.').next().unwrap_or("")))
    {
        "es un nombre reservado de Windows"
    } else {
        return Ok(());
    };
    Err(Error::Invalid(format!(
        "Nombre inválido '{nombre}': {razon}"
    )))
}
