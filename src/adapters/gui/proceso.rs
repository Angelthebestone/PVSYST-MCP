//! PVsyst GUI process execution, probing and project path confinement.

use super::escritorio;
use super::ffi::{self, enum_windows, pids_de_ejecutable};
use super::ventanas::{class_of, enum_windows_de, pid_of, ventanas, visible, window_brief};
use crate::args::flag;
use crate::error::{Error, Result};
use crate::workspace::{resolver_ruta, Alcance};
use serde_json::{json, Value};
use std::path::{Path, PathBuf};

const DEFAULT_LAUNCH_WAIT_SECS: u64 = 30;
const MAX_LAUNCH_WAIT_SECS: u64 = 120;
const AVISO_DEMO: &str = "TDEMOForm";
const POLL_INTERVAL_MS: u64 = 250;

pub(crate) fn gui_running() -> bool {
    !pids_de_ejecutable("PVsyst.exe").is_empty()
}

/// `gui_actuar accion:cerrar_todo`: closes every visible top window of the
/// running PVsyst. It presses Cancelar/No before the default buttons unless the call
/// passes `cancelar:false`, so a save prompt is never accepted by accident.
pub(crate) fn cerrar_todo(_paths: &crate::config::Paths, args: &Value) -> Result<Value> {
    let pids = pids_de_ejecutable("PVsyst.exe");
    let ventanas_vivas: Vec<ffi::Hwnd> = pids
        .iter()
        .flat_map(|p| enum_windows_de(*p))
        .filter(|h| visible(*h))
        .collect();
    let mut con_cancelar = args.clone();
    if let Some(argumentos) = con_cancelar.as_object_mut() {
        argumentos.entry("cancelar").or_insert(json!(true));
    }
    for h in &ventanas_vivas {
        super::acciones::cerrar_ventana(*h, &con_cancelar)?;
    }
    Ok(json!({
        "accion": "cerrar_todo",
        "pids": pids,
        "cerradas": ventanas_vivas.len(),
    }))
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

/// Closes the demo-expiry notice of process `pid` when `cerrar_demo:true`.
fn cerrar_aviso_demo_si_pedido(pid: u32, args: &Value) {
    if !flag(args, &["cerrar_demo"]) {
        return;
    }
    enum_windows_de(pid)
        .into_iter()
        .filter(|h| class_of(*h) == AVISO_DEMO)
        .for_each(|h| {
            ffi::post_message(h, ffi::Mensaje::simple(ffi::WM_CLOSE));
        });
}

/// `proyecto` cannot open into a running PVsyst: the request is refused by name.
fn comprobar_proyecto_libre(args: &Value) -> Result<()> {
    if args.get("proyecto").is_none() {
        return Ok(());
    }
    Err(Error::Gui(
        "'proyecto' no se puede abrir: PVsyst ya está en ejecución. Ciérrelo (gui_actuar accion:cerrar_todo) y repita lanzar con proyecto".into(),
    ))
}

/// PVsyst allows one instance per session: a second launch stops at the box
/// "PVsyst is already running." and never opens. So `lanzar` attaches to the
/// running instance instead of spawning another one.
fn ya_abierta(paths: &crate::config::Paths, args: &Value) -> Result<Value> {
    comprobar_proyecto_libre(args)?;
    let pid = pids_de_ejecutable("PVsyst.exe")
        .first()
        .copied()
        .unwrap_or(0);
    cerrar_aviso_demo_si_pedido(pid, args);
    let windows = ventanas(paths, &json!({ "visibles": true }))?;
    Ok(json!({
        "pid": pid,
        "gui": paths.gui_exe.to_string_lossy(),
        "ready": esperar_formulario_principal(pid, 0),
        "ya_en_ejecucion": true,
        "windows": windows,
    }))
}

pub(crate) fn lanzar(paths: &crate::config::Paths, args: &Value) -> Result<Value> {
    if !paths.gui_exe.exists() {
        return Err(Error::Gui(format!(
            "PVsyst GUI not found at '{}'. Set PVSYST_GUI.",
            paths.gui_exe.display()
        )));
    }
    if gui_running() {
        return ya_abierta(paths, args);
    }
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
    cerrar_aviso_demo_si_pedido(pid, args);
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
