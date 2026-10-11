//! PVsyst GUI process execution, probing and project path confinement.

use super::ffi::enum_windows;
use super::ventanas::{find_pids, pid_of, ventanas, visible, window_brief};
use crate::error::{Error, Result};
use crate::workspace::{resolver_ruta, Alcance};
use serde_json::{json, Value};
use std::path::PathBuf;

const DEFAULT_LAUNCH_WAIT_SECS: u64 = 30;
const MAX_LAUNCH_WAIT_SECS: u64 = 120;
const POLL_INTERVAL_MS: u64 = 250;

pub(crate) fn gui_running() -> bool {
    !find_pids("PVsyst.exe").is_empty()
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
        enum_windows()
            .into_iter()
            .any(|h| pid_of(h) == pid && visible(h) && window_brief(h)["class"] == "THomeForm")
    };
    while !ready() && std::time::Instant::now() < deadline {
        std::thread::sleep(std::time::Duration::from_millis(POLL_INTERVAL_MS));
    }
    ready()
}

pub(crate) fn lanzar(paths: &crate::config::Paths, args: &Value) -> Result<Value> {
    if !paths.gui_exe.exists() {
        return Err(Error::Gui(format!(
            "PVsyst GUI not found at '{}'. Set PVSYST_GUI.",
            paths.gui_exe.display()
        )));
    }
    let mut cmd = std::process::Command::new(&paths.gui_exe);
    if let Some(p) = args.get("proyecto").and_then(|v| v.as_str()) {
        let path = resolver_proyecto(paths, args, p)?;
        cmd.arg(path);
    }
    let child = cmd
        .spawn()
        .map_err(|e| Error::Gui(format!("Failed to launch PVsyst GUI: {e}")))?;
    let pid = child.id();
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
mod tests {
    use super::*;
    use std::sync::Arc;

    #[test]
    fn should_reject_project_outside_workspace_by_default() {
        let ws = std::env::temp_dir().join("test_pvsyst_prj_confined");
        let paths = crate::config::Paths {
            workspace: ws,
            cli_exe: PathBuf::new(),
            gui_exe: PathBuf::new(),
            experto: false,
            estado: Arc::default(),
        };
        let args = json!({});
        let res = resolver_proyecto(&paths, &args, "../../secret.PRJ");
        assert!(res.is_err(), "Must reject escaping project path by default");
    }
}
