//! Detección de caídas de PVsystCLI en sus logs Admin.
//!
//! PVsystCLI termina con código 0 aunque el hilo que ejecuta el comando muera por
//! una excepción: la única evidencia es la línea que escribe en su log Admin,
//! medida en PVsyst 8.1.8 (rev. 49527):
//!
//! ```text
//! 01:58:59|Err |EPrivilege      |run-simulation      |Instruction privilégiée
//! ```
//!
//! Sin esa lectura, una simulación caída se clasificaba como «ambiguo» y el agente
//! la reintentaba gastando una ejecución TRIAL en cada intento.

use std::fs;
use std::path::{Path, PathBuf};
use std::time::{Duration, SystemTime};

/// Clase de excepción que PVsystCLI registra cuando muere su hilo de trabajo.
const CLASE_CRASH: &str = "EPrivilege";

/// Margen para el desfase entre la hora del log y la del reloj del servidor.
const MARGEN: Duration = Duration::from_secs(2);

/// Mensaje de la primera caída registrada en los logs Admin desde `desde`.
pub(crate) fn crash_desde(desde: SystemTime) -> Option<String> {
    let dir = crate::config::local_app_data()
        .ok()?
        .join("PVsystCLI")
        .join("8.1")
        .join("Admin");
    crash_en_directorio(&dir, desde)
}

/// Same as [`crash_desde`] over an explicit log folder, so it can be tested.
pub(crate) fn crash_en_directorio(dir: &Path, desde: SystemTime) -> Option<String> {
    let umbral = desde.checked_sub(MARGEN).unwrap_or(desde);
    let mut archivos: Vec<(SystemTime, PathBuf)> = fs::read_dir(dir)
        .ok()?
        .flatten()
        .filter_map(|entrada| {
            let ruta = entrada.path();
            if !ruta.to_str().is_some_and(|n| n.ends_with(".log")) {
                return None;
            }
            // Each run opens its own log: the creation time is when that process started.
            // The modification time would also match the log of a command that crashed
            // a moment before this one and blame this run for it.
            let meta = entrada.metadata().ok()?;
            let inicio = meta.created().or_else(|_| meta.modified()).ok()?;
            (inicio >= umbral).then_some((inicio, ruta))
        })
        .collect();
    archivos.sort_by_key(|(modificado, _)| *modificado);
    archivos.iter().find_map(|(_, ruta)| crash_en_archivo(ruta))
}

fn crash_en_archivo(ruta: &Path) -> Option<String> {
    let bytes = fs::read(ruta).ok()?;
    let texto = crate::format::decodificar_texto_flexible(&bytes);
    texto
        .lines()
        .find(|l| l.contains(CLASE_CRASH))
        .map(mensaje_de)
        .filter(|m| !m.is_empty())
}

/// Última columna de una línea de log de PVsystCLI, que es el mensaje.
fn mensaje_de(linea: &str) -> String {
    linea.rsplit('|').next().unwrap_or(linea).trim().to_string()
}

#[cfg(test)]
#[path = "../../../tests/unit/adapters/cli/crash.rs"]
mod tests;
