//! Environment diagnostics and installation readiness checks.

use crate::config::{self, Paths};
use serde_json::{json, Value};
use std::path::{Path, PathBuf};

/// Result of one readiness check.
#[derive(Clone, Copy)]
enum EstadoCheck {
    Correcto,
    Fallido,
}

impl EstadoCheck {
    fn de_ruta(ruta: &Path) -> Self {
        if ruta.exists() {
            Self::Correcto
        } else {
            Self::Fallido
        }
    }
}

/// One readiness check, ready to be rendered for the caller.
struct Comprobacion<'a> {
    nombre: &'a str,
    estado: EstadoCheck,
    detalle: String,
    pista: &'a str,
}

impl<'a> Comprobacion<'a> {
    fn de_ruta(nombre: &'a str, ruta: &Path, pista: &'a str) -> Self {
        Self {
            nombre,
            estado: EstadoCheck::de_ruta(ruta),
            detalle: ruta.display().to_string(),
            pista,
        }
    }

    fn a_json(&self) -> Value {
        json!({
            "check": self.nombre,
            "ok": matches!(self.estado, EstadoCheck::Correcto),
            "detail": self.detalle,
            "hint": self.pista,
        })
    }
}

fn instalacion(paths: &Paths) -> Value {
    Comprobacion::de_ruta(
        "pvsyst_install",
        &paths.gui_exe,
        "Install PVsyst 8.x (licencia legítima) o define PVSYST_GUI.",
    )
    .a_json()
}

fn cli(paths: &Paths) -> Value {
    Comprobacion::de_ruta(
        "cli",
        &paths.cli_exe,
        "Se necesita PVsystCLI.exe para simular sin GUI; define PVSYST_CLI.",
    )
    .a_json()
}

fn workspace(paths: &Paths) -> Value {
    Comprobacion::de_ruta(
        "workspace",
        &paths.workspace,
        "Crea el workspace (o define PVSYST_WORKSPACE). PVsyst lo crea al primer arranque.",
    )
    .a_json()
}

fn projects_dir(paths: &Paths) -> Value {
    let projects = paths.workspace.join("Projects");
    Comprobacion::de_ruta(
        "projects_dir",
        &projects,
        "Hace falta la carpeta Projects\\ dentro del workspace.",
    )
    .a_json()
}

fn weather_grids() -> Value {
    let nasa = config::data_ro().join("Global_World.txt");
    Comprobacion::de_ruta(
        "weather_grids",
        &nasa,
        "Las rejillas NASA permiten crear sitios solo con coordenadas.",
    )
    .a_json()
}

fn cli_license() -> Value {
    let lic = PathBuf::from(r"C:\ProgramData\PVsystCLI\PVsystCLI.lic");
    Comprobacion::de_ruta(
        "cli_license",
        &lic,
        "Sin licencia el CLI funciona en modo TRIAL (dias y ejecuciones limitados); mira el estado con la herramienta 'entorno' (accion 'licencia') y activa con 'entorno' accion 'licencia_activar'.",
    )
    .a_json()
}

/// Installation, workspace and licence checks, each with a hint for fixing it.
pub(crate) fn comprobaciones(paths: &Paths) -> Vec<Value> {
    vec![
        instalacion(paths),
        cli(paths),
        workspace(paths),
        projects_dir(paths),
        weather_grids(),
        cli_license(),
    ]
}

/// Environment readiness with guidance for non-experts.
pub(crate) fn preflight(paths: &Paths) -> Value {
    let checks = comprobaciones(paths);
    let all_ok = checks
        .iter()
        .filter(|c| c["check"] != "cli_license")
        .all(|c| {
            c.get("ok")
                .and_then(serde_json::Value::as_bool)
                .unwrap_or(false)
        });
    json!({
        "ok": all_ok,
        "workspace": paths.workspace.display().to_string(),
        "cli_exe": paths.cli_exe.display().to_string(),
        "gui_exe": paths.gui_exe.display().to_string(),
        "checks": checks,
        "next": if all_ok {
            "Todo listo: usa 'disenar_sistema' o lee el recurso pvsyst://recetas para empezar."
        } else {
            "Corrige los checks marcados como false; mira el campo 'hint'."
        },
        "tip": "Para licencia usa la herramienta 'entorno'; para comandos CLI avanzados 'pvsyst_cli'."
    })
}
