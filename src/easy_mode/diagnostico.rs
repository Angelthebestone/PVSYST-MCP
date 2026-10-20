//! Environment diagnostics and installation readiness checks.

use crate::config::{self, Paths};
use serde_json::{json, Value};
use std::path::PathBuf;

/// Installation, workspace and licence checks, each with a hint for fixing it.
pub(crate) fn comprobaciones(paths: &Paths) -> Vec<Value> {
    let mut checks: Vec<Value> = Vec::new();
    let mut push = |name: &str, ok: bool, detail: String, hint: &str| {
        checks.push(json!({ "check": name, "ok": ok, "detail": detail, "hint": hint }));
    };
    push(
        "pvsyst_install",
        paths.gui_exe.exists(),
        paths.gui_exe.display().to_string(),
        "Install PVsyst 8.1.7 (licencia legítima) o define PVSYST_GUI.",
    );
    push(
        "cli",
        paths.cli_exe.exists(),
        paths.cli_exe.display().to_string(),
        "Se necesita PVsystCLI.exe para simular sin GUI; define PVSYST_CLI.",
    );
    push(
        "workspace",
        paths.workspace.exists(),
        paths.workspace.display().to_string(),
        "Crea el workspace (o define PVSYST_WORKSPACE). PVsyst lo crea al primer arranque.",
    );
    push(
        "projects_dir",
        paths.workspace.join("Projects").exists(),
        paths.workspace.join("Projects").display().to_string(),
        "Hace falta la carpeta Projects\\ dentro del workspace.",
    );
    let nasa = config::data_ro().join("Global_World.txt");
    push(
        "weather_grids",
        nasa.exists(),
        nasa.display().to_string(),
        "Las rejillas NASA permiten crear sitios solo con coordenadas.",
    );
    let lic = PathBuf::from(r"C:\ProgramData\PVsystCLI\PVsystCLI.lic");
    push(
        "cli_license",
        lic.exists(),
        lic.display().to_string(),
        "Sin licencia el CLI funciona en modo TRIAL (dias y ejecuciones limitados); mira el estado con la herramienta 'entorno' (accion 'licencia') y activa con 'entorno' accion 'licencia_activar'.",
    );
    checks
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
