//! GUI bridge: drive the PVsyst desktop application (`PVsyst.exe`) through the
//! Windows API — the only way to cover PVsyst features that have **no file or
//! CLI backing** (3D shading editor, informative I/V & table tools, import
//! wizards, dialogs...).
//!
//! This is black-box UI automation: it enumerates real Win32 windows/controls
//! (VCL forms are native windows), reads the UI Automation tree (`uia`) and
//! posts standard window messages to invoke them. It never reads, disassembles
//! or patches PVsyst code. Captures are PNG; physical clicks and captures need
//! an interactive desktop.

pub(crate) mod acciones;
pub(crate) mod base64;
pub(crate) mod captura;
pub(crate) mod esperas;
pub(crate) mod ffi;
pub(crate) mod gdi;
pub(crate) mod png;
pub(crate) mod proceso;
pub(crate) mod teclado;
pub(crate) mod uia_vista;
pub(crate) mod ventanas;

pub(crate) use proceso::gui_running;

use crate::error::{Error, Result};
use serde_json::Value;

/// `observar` dispatcher. `que`: ventanas (default) | controles | leer |
/// capturar | uia | arbol.
pub(crate) fn observar(paths: &crate::config::Paths, args: &Value) -> Result<Value> {
    match args
        .get("que")
        .and_then(Value::as_str)
        .unwrap_or("ventanas")
    {
        "ventanas" => ventanas::ventanas(paths, args),
        "controles" => uia_vista::controles_con_uia(paths, args),
        "leer" => ventanas::leer(paths, args),
        "capturar" => captura::capturar(paths, args),
        "uia" => uia_vista::observar_uia(paths, args),
        "arbol" => uia_vista::observar_arbol(paths, args),
        other => Err(Error::Invalid(format!(
            "Unknown que '{other}' (ventanas|controles|leer|capturar|uia|arbol)"
        ))),
    }
}

/// `actuar` dispatcher. `accion`: click | clic_fisico | mensaje | cerrar |
/// activar | teclas | lanzar | uia_invocar | esperar | paso.
pub(crate) fn actuar(paths: &crate::config::Paths, args: &Value) -> Result<Value> {
    match args.get("accion").and_then(Value::as_str).unwrap_or("click") {
        accion @ ("click" | "mensaje" | "cerrar" | "activar") => {
            acciones::invocar(paths, args).map_err(|e| Error::Gui(format!("{e} (accion {accion})")))
        }
        "clic_fisico" => {
            acciones::exigir_escritorio()?;
            acciones::invocar(paths, args)
        }
        "teclas" => teclado::teclas(paths, args),
        "lanzar" => proceso::lanzar(paths, args),
        "uia_invocar" => uia_vista::uia_invocar(args),
        "esperar" => esperas::esperar(args),
        "paso" => uia_vista::paso(paths, args),
        other => Err(Error::Invalid(format!(
            "Unknown accion '{other}' (click|clic_fisico|mensaje|cerrar|activar|teclas|lanzar|uia_invocar|esperar|paso)"
        ))),
    }
}
