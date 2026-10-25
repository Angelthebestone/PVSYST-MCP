//! PVsyst GUI process execution, probing and project path confinement.

use super::escritorio;
use super::ffi::{enum_windows, pids_de_ejecutable};
use super::ventanas::{pid_of, ventanas, visible, window_brief};
use crate::error::{Error, Result};
use crate::workspace::{resolver_ruta, Alcance};
use serde_json::{json, Value};
use std::path::{Path, PathBuf};

const DEFAULT_LAUNCH_WAIT_SECS: u64 = 30;
const MAX_LAUNCH_WAIT_SECS: u64 = 120;
const POLL_INTERVAL_MS: u64 = 250;

pub(crate) fn gui_running() -> bool {
    !pids_de_ejecutable("PVsyst.exe").is_empty()
}

pub(crate) fn resolver_proyecto(
    paths: &crate::config::Paths,
    args: &Value,
    p: &str,
) -> Result<PathBuf> {
    let alcance = Alcance::de_args(args);
    let cand_prj = format!("Projects/{p}.PRJ");
    if paths.workspace.join(&cand_prj).exists() {
        return resolver_ruta(&paths.workspace, &cand_prj, alcance);
    }
    let cand_raw = format!("Projects/{p}");
    if paths.workspace.join(&cand_raw).exists() {
        return resolver_ruta(&paths.workspace, &cand_raw, alcance);
    }
    resolver_ruta(&paths.workspace, p, alcance)
}

fn esperar_formulario_principal(pid: u32, timeout_secs: u64) -> bool {
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(timeout_secs);
    let ready = || {
        enum_windows().into_iter().any(|h| {
            pid_of(h) == pid
                && visible(h)
                && window_brief(h).get("class").and_then(Value::as_str) == Some("THomeForm")
        })
    };
    while !ready() && std::time::Instant::now() < deadline {
        std::thread::sleep(std::time::Duration::from_millis(POLL_INTERVAL_MS));
    }
    ready()
}

/// Starts PVsyst (with `proyecto` when given) on the right desktop and returns
/// its process id.
fn iniciar_proceso(exe: &Path, proyecto: Option<&Path>) -> Result<u32> {
    if escritorio::es_aislado() {
        let args: Vec<&Path> = proyecto.into_iter().collect();
        return escritorio::lanzar_proceso(exe, &args, exe.parent());
    }
    let mut cmd = std::process::Command::new(exe);
    if let Some(p) = proyecto {
        cmd.arg(p);
    }
    cmd.spawn()
        .map(|child| child.id())
        .map_err(|e| Error::Gui(format!("Failed to launch PVsyst GUI: {e}")))
}

/// PVsyst allows one instance per session: a second one, even on the private
/// desktop, stops at the box "PVsyst is already running." and never opens. So
/// on the private desktop a running PVsyst (anywhere) is an error up front.
fn exigir_sin_otra_instancia() -> Result<()> {
    if !escritorio::es_aislado() {
        return Ok(());
    }
    match pids_de_ejecutable("PVsyst.exe").first() {
        None => Ok(()),
        Some(pid) => Err(Error::Gui(format!(
            "PVsyst ya está en ejecución (pid {pid}): con PVSYST_MCP_ESCRITORIO=aislado una segunda instancia se detiene en «PVsyst is already running». Ciérralo, o usa el escritorio compartido para manejar esa instancia"
        ))),
    }
}

pub(crate) fn lanzar(paths: &crate::config::Paths, args: &Value) -> Result<Value> {
    if !paths.gui_exe.exists() {
        return Err(Error::Gui(format!(
            "PVsyst GUI not found at '{}'. Set PVSYST_GUI.",
            paths.gui_exe.display()
        )));
    }
    exigir_sin_otra_instancia()?;
    let proyecto = match args.get("proyecto").and_then(|v| v.as_str()) {
        Some(p) => Some(resolver_proyecto(paths, args, p)?),
        None => None,
    };
    let pid = iniciar_proceso(&paths.gui_exe, proyecto.as_deref())?;
    let wait = args
        .get("esperar_segundos")
        .and_then(serde_json::Value::as_u64)
        .unwrap_or(DEFAULT_LAUNCH_WAIT_SECS)
        .min(MAX_LAUNCH_WAIT_SECS);
    let ready = esperar_formulario_principal(pid, wait);
    let wins = ventanas(paths, &json!({ "visibles": true }))?;
    Ok(json!({
        "pid": pid,
        "gui": paths.gui_exe.to_string_lossy(),
        "ready": ready,
        "windows": wins,
    }))
}

#[cfg(test)]
#[path = "../../../tests/unit/adapters/gui/proceso.rs"]
mod tests;
