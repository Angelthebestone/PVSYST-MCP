use crate::config::Paths;
use crate::error::{Error, Result};
use serde_json::{json, Value};
use std::path::Path;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum ModoGui {
    Abrir,
    NoAbrir,
}

impl ModoGui {
    pub(crate) fn de_bool(abrir: bool) -> Self {
        if abrir {
            Self::Abrir
        } else {
            Self::NoAbrir
        }
    }

    pub(crate) fn debe_abrir(self) -> bool {
        self == Self::Abrir
    }
}

pub(crate) fn spawn_gui(paths: &Paths, prj: &Path) -> Result<Value> {
    std::process::Command::new(&paths.gui_exe)
        .arg(prj)
        .spawn()
        .map(|child| json!({ "pid": child.id(), "gui": paths.gui_exe.to_string_lossy() }))
        .map_err(|e| {
            Error::io(
                format!("Failed to launch PVsyst GUI '{}'", paths.gui_exe.display()),
                e,
            )
        })
}

pub(crate) fn sin_cli(
    paths: &Paths,
    prj_path: &Path,
    stem: &str,
    modo_gui: ModoGui,
) -> Result<Value> {
    if !modo_gui.debe_abrir() {
        return Err(Error::Cli(format!(
            "PVsystCLI.exe not found at '{}'. Set PVSYST_CLI, or call with abrir_gui=true to \
             leave the project ready for the PVsyst GUI.",
            paths.cli_exe.display()
        )));
    }
    let spawn = spawn_gui(paths, prj_path)?;
    Ok(json!({
        "mode": "gui",
        "success": true,
        "message": "PVsystCLI.exe not found; launched the PVsyst GUI with the project ready to run.",
        "project": stem,
        "spawn": spawn,
    }))
}
