//! PVsyst GUI process execution, probing and project path confinement.

use super::escritorio;
use super::ffi::{self, enum_windows, pids_de_ejecutable};
use super::ventanas::{
    class_of, enum_windows_de, pid_of, ventanas, visible, window_brief, window_json,
};
use crate::args::flag;
use crate::error::{Error, Result};
use crate::idioma::{Idioma, Texto};
use crate::workspace::{resolver_ruta, Alcance};
use serde_json::{json, Value};
use std::path::{Path, PathBuf};

const DEFAULT_LAUNCH_WAIT_SECS: u64 = 30;
const MAX_LAUNCH_WAIT_SECS: u64 = 120;
const AVISO_DEMO: &str = "TDEMOForm";
const POLL_INTERVAL_MS: u64 = 250;
/// Closing an editor can reopen its parent, so the list is read again each round.
const MAX_RONDAS_CIERRE: usize = 12;
const ESPERA_RONDA: std::time::Duration = std::time::Duration::from_millis(300);

/// Processes of the PVsyst executable currently running.
pub(crate) fn pids_pvsyst() -> Vec<u32> {
    pids_de_ejecutable("PVsyst.exe")
}

pub(crate) fn gui_running() -> bool {
    !pids_pvsyst().is_empty()
}

/// `gui_actuar accion:cerrar_todo`: closes every visible top window of the
/// running PVsyst. It presses Cancelar/No before the default buttons unless the call
/// passes `cancelar:false`, so a save prompt is never accepted by accident.
fn visibles_de(pids: &[u32]) -> Vec<ffi::Hwnd> {
    pids.iter()
        .flat_map(|p| enum_windows_de(*p))
        .filter(|h| visible(*h))
        .collect()
}

/// Whether another round is needed: the list is re-read after each one, because
/// closing an editor can reopen the selector it came from.
fn otra_ronda(quedan: usize, rondas: usize) -> bool {
    quedan > 0 && rondas < MAX_RONDAS_CIERRE
}

/// Adds the handles of `quedan` that the round confirmed gone, once each.
fn anotar_cerradas(quedan: &[ffi::Hwnd], vivas: &[ffi::Hwnd], cerradas: &mut Vec<ffi::Hwnd>) {
    for h in quedan {
        if vivas.contains(h) || cerradas.contains(h) {
            continue;
        }
        cerradas.push(*h);
    }
}

pub(crate) fn cerrar_todo(args: &Value) -> Result<Value> {
    let pids = pids_pvsyst();
    let mut con_cancelar = args.clone();
    if let Some(argumentos) = con_cancelar.as_object_mut() {
        argumentos.entry("cancelar").or_insert(json!(true));
    }
    // A window that survives a round is retried in the next one: count each handle
    // once, when it is confirmed gone, instead of counting the attempts.
    let mut cerradas: Vec<ffi::Hwnd> = Vec::new();
    let mut rondas = 0;
    let mut quedan = visibles_de(&pids);
    while otra_ronda(quedan.len(), rondas) {
        for h in &quedan {
            super::acciones::cerrar_ventana(*h, &con_cancelar)?;
        }
        rondas += 1;
        std::thread::sleep(ESPERA_RONDA);
        let vivas = visibles_de(&pids);
        anotar_cerradas(&quedan, &vivas, &mut cerradas);
        // A round that closed no window will not close one in the next round either.
        let cerro_alguna = quedan.iter().any(|h| !vivas.contains(h));
        quedan = vivas;
        if !cerro_alguna {
            break;
        }
    }
    Ok(json!({
        "accion": "cerrar_todo",
        "pids": pids,
        "cerradas": cerradas.len(),
        "rondas": rondas,
        "quedan": quedan.iter().map(|h| window_json(*h)).collect::<Vec<_>>(),
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
fn comprobar_proyecto_libre(args: &Value, idioma: Idioma) -> Result<()> {
    if args.get("proyecto").is_none() {
        return Ok(());
    }
    let ya_en_ejecucion = Texto {
        es: "'proyecto' no se puede abrir: PVsyst ya está en ejecución. Ciérralo (gui_actuar accion:cerrar_todo) y repite lanzar con proyecto",
        en: "'proyecto' cannot be opened: PVsyst is already running. Close it (gui_actuar accion:cerrar_todo) and repeat lanzar with proyecto",
    };
    Err(Error::Gui(ya_en_ejecucion.en_idioma(idioma).into()))
}

/// PVsyst allows one instance per session: a second launch stops at the box
/// "PVsyst is already running." and never opens. So `lanzar` attaches to the
/// running instance instead of spawning another one.
fn ya_abierta(paths: &crate::config::Paths, args: &Value) -> Result<Value> {
    comprobar_proyecto_libre(args, paths.idioma)?;
    let pid = pids_pvsyst().first().copied().unwrap_or(0);
    cerrar_aviso_demo_si_pedido(pid, args);
    let windows = ventanas(&json!({ "visibles": true }))?;
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
        let no_esta = Texto {
            es: "No se encontró la GUI de PVsyst en '{ruta}'. Define PVSYST_GUI.",
            en: "PVsyst GUI not found at '{ruta}'. Set PVSYST_GUI.",
        };
        return Err(Error::Gui(no_esta.texto_con(
            paths.idioma,
            &[("ruta", &paths.gui_exe.display().to_string())],
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
    let wins = ventanas(&json!({ "visibles": true }))?;
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
