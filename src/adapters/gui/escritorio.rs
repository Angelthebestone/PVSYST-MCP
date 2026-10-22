//! Where PVsyst's windows live: the user's desktop (default) or a private one.
//!
//! `PVSYST_MCP_ESCRITORIO=aislado` makes the MCP create its own Win32 desktop
//! and run PVsyst there, so the agent can drive the GUI without taking the
//! user's focus, cursor or keyboard. Everything it "types" is posted to the
//! windows as messages (see `ffi::escritorio`).

use super::ffi;
use crate::error::{Error, Result};
use std::path::Path;
use std::sync::OnceLock;

/// Environment variable that selects the mode.
pub(crate) const VARIABLE: &str = "PVSYST_MCP_ESCRITORIO";
/// Name of the private desktop.
const NOMBRE: &str = "pvsyst_mcp_agente";

/// Which desktop the MCP drives.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Modo {
    /// The user's own desktop (needs an interactive session).
    Compartido,
    /// A private desktop owned by the MCP.
    Aislado,
}

/// Handle del escritorio privado Win32 (HDESK); creado una vez al inicio del proceso y reutilizado.
///
/// Decisión de diseño (§4b.5): Un contexto explícito no es viable aquí porque `HDESK` es un recurso
/// a nivel de proceso del sistema operativo Win32. Múltiples hilos de trabajo asíncronos necesitan
/// adjuntarse (`SetThreadDesktop`) antes de invocar APIs de ventanas y UIA sin acoplar firmas en todo
/// el crate. `OnceLock` garantiza inicialización segura, concurrente e inmutable una sola vez.
static ESCRITORIO: OnceLock<isize> = OnceLock::new();

/// Parses the value of [`VARIABLE`]: absent, empty or `compartido` is the
/// user's desktop, `aislado` the private one; anything else is an error so a
/// typo never silently falls back to driving the user's desktop.
pub(crate) fn modo_de(valor: Option<&str>) -> Result<Modo> {
    match valor.map(|v| v.trim().to_ascii_lowercase()).as_deref() {
        None | Some("") | Some("compartido") => Ok(Modo::Compartido),
        Some("aislado") => Ok(Modo::Aislado),
        Some(otro) => Err(Error::Invalid(format!(
            "{VARIABLE}='{otro}' no es válido: usa 'compartido' o 'aislado'"
        ))),
    }
}

/// Reads [`VARIABLE`] and, for `aislado`, creates the desktop and switches the
/// GUI adapter to posted input. Call once at start-up.
pub(crate) fn preparar() -> Result<Modo> {
    let modo = modo_de(std::env::var(VARIABLE).ok().as_deref())?;
    if modo == Modo::Compartido {
        ffi::fijar_aislado(ffi::ModoAislamiento::Compartido);
        return Ok(modo);
    }
    let h = ffi::crear_escritorio(NOMBRE).map_err(|e| {
        Error::Gui(format!(
            "No se pudo crear el escritorio aislado (Win32 {e})"
        ))
    })?;
    // Un segundo `preparar` reutiliza el handle ya creado.
    let _ = ESCRITORIO.set(h);
    ffi::fijar_aislado(ffi::ModoAislamiento::Aislado);
    Ok(modo)
}

/// Attaches the calling thread to the private desktop, when there is one. It
/// must run before the thread creates any window, hook or UI Automation
/// client, so the tool worker calls it first.
pub(crate) fn adjuntar_hilo_actual() -> Result<()> {
    let Some(h) = ESCRITORIO.get() else {
        return Ok(());
    };
    ffi::adjuntar_hilo(*h)
        .map_err(|e| Error::Gui(format!("No se pudo usar el escritorio aislado (Win32 {e})")))
}

/// Whether the MCP works on its private desktop.
pub(crate) fn es_aislado() -> bool {
    ffi::aislado()
}

/// Name of the mode, for diagnostics.
pub(crate) fn nombre_del_modo() -> &'static str {
    if es_aislado() {
        "aislado"
    } else {
        "compartido"
    }
}

/// Command line for `exe` and `args`, quoted for `CreateProcessW`.
pub(crate) fn linea_de_comando(exe: &Path, args: &[&Path]) -> String {
    std::iter::once(exe)
        .chain(args.iter().copied())
        .map(|p| format!("\"{}\"", p.display()))
        .collect::<Vec<_>>()
        .join(" ")
}

/// Starts `exe` on the private desktop and returns its process id.
pub(crate) fn lanzar_proceso(exe: &Path, args: &[&Path], directorio: Option<&Path>) -> Result<u32> {
    let linea = linea_de_comando(exe, args);
    let directorio = directorio.map(|d| d.display().to_string());
    ffi::lanzar_en_escritorio(&linea, directorio.as_deref(), NOMBRE).map_err(|e| {
        Error::Gui(format!(
            "No se pudo lanzar PVsyst en el escritorio aislado (Win32 {e})"
        ))
    })
}

#[cfg(test)]
#[path = "../../../tests/unit/adapters/gui/escritorio.rs"]
mod tests;
