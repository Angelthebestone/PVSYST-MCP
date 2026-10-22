//! `gui_actuar accion:reiniciar`: gets PVsyst back to a clean home screen after
//! it failed. It acts only on the two failure screens PVsyst itself shows (the
//! madExcept crash dialog and the "unstable after a crash" warning), so it
//! can never discard work PVsyst did not already declare at risk.

use super::acciones::{clic_por_mensajes, CLASE_EXCEPCION, CLASE_MENSAJE};
use super::conduccion::esperar;
use super::ffi::Hwnd;
use super::ocr::captura_ocr::lineas_vivas;
use super::ocr::ubicacion::plegar;
use super::proceso::lanzar;
use super::rotulos::{rotulos_recuperacion, Idioma, RotulosRecuperacion};
use super::ventanas::{
    class_of, enum_windows_de, find_pids, hijo_visible_con_texto, target_pids, visible,
};
use crate::error::{Error, Result};
use serde_json::{json, Value};
use std::time::{Duration, Instant};

const INICIO: &str = "THomeForm";
const EJECUTABLE: &str = "PVsyst.exe";
const ROTULOS: RotulosRecuperacion = rotulos_recuperacion(Idioma::Es);
const ESPERA_SALIDA: Duration = Duration::from_secs(20);
const ESPERA_ARRANQUE: Duration = Duration::from_secs(60);
const SONDEO: Duration = Duration::from_millis(250);

/// Which failure screen PVsyst is showing.
enum Fallo {
    /// The madExcept crash dialog: its own button restarts PVsyst.
    Excepcion(Hwnd),
    /// The warning after a crash: answering "No" quits PVsyst.
    Inestable(Hwnd),
}

fn es_aviso_inestable(ventana: Hwnd) -> bool {
    hijo_visible_con_texto(ventana, ROTULOS.boton_salir).is_some()
        && lineas_vivas(ventana).is_ok_and(|lineas| {
            let texto = plegar(&lineas.iter().map(|l| l.texto.as_str()).collect::<String>());
            ROTULOS.aviso_inestable.iter().any(|p| texto.contains(p))
        })
}

fn fallo_de(pid: u32) -> Option<Fallo> {
    enum_windows_de(pid)
        .into_iter()
        .filter(|h| visible(*h))
        .find_map(|h| match class_of(h).as_str() {
            CLASE_EXCEPCION => Some(Fallo::Excepcion(h)),
            CLASE_MENSAJE if es_aviso_inestable(h) => Some(Fallo::Inestable(h)),
            _ => None,
        })
}

/// Waits until process `pid` is gone; `false` on timeout.
fn esperar_salida(pid: u32) -> bool {
    let fin = Instant::now() + ESPERA_SALIDA;
    while find_pids(EJECUTABLE).contains(&pid) {
        if Instant::now() >= fin {
            return false;
        }
        std::thread::sleep(SONDEO);
    }
    true
}

/// What starts after PVsyst quits.
enum Reaparicion {
    /// A new process with its home screen up.
    Lista(u32),
    /// A new process exists but its home screen never came up.
    SinInicio(u32),
    /// No new process within the launch budget.
    Ninguna,
}

fn nueva_instancia(anterior: u32) -> Reaparicion {
    let fin = Instant::now() + ESPERA_ARRANQUE;
    let mut nuevo = None;
    while Instant::now() < fin {
        let vivos = find_pids(EJECUTABLE);
        // A process that died before showing its home screen no longer counts.
        nuevo = nuevo
            .filter(|p| vivos.contains(p))
            .or_else(|| vivos.into_iter().find(|p| *p != anterior));
        if let Some(pid) = nuevo.filter(|&p| esperar(p, INICIO, SONDEO * 4).is_ok()) {
            return Reaparicion::Lista(pid);
        }
        std::thread::sleep(SONDEO);
    }
    nuevo.map_or(Reaparicion::Ninguna, Reaparicion::SinInicio)
}

fn pulsar(ventana: Hwnd, rotulo: &str) -> Result<()> {
    let boton = hijo_visible_con_texto(ventana, rotulo)
        .ok_or_else(|| Error::Gui(format!("El diálogo de fallo no ofrece '{rotulo}'")))?;
    clic_por_mensajes(boton)
}

/// How a failure screen is left, and whether PVsyst starts itself again after.
struct Salida {
    via: &'static str,
    pvsyst_se_reinicia: bool,
}

fn salir_del_fallo(fallo: &Fallo) -> Result<Salida> {
    match *fallo {
        Fallo::Excepcion(h) => pulsar(h, ROTULOS.boton_reiniciar).map(|()| Salida {
            via: "madExcept",
            pvsyst_se_reinicia: true,
        }),
        Fallo::Inestable(h) => pulsar(h, ROTULOS.boton_salir).map(|()| Salida {
            via: "aviso_inestable",
            pvsyst_se_reinicia: false,
        }),
    }
}

/// The process showing a failure screen, and the screen.
fn proceso_en_fallo(args: &Value) -> Result<(u32, Fallo)> {
    let pids = target_pids(args)?;
    if pids.is_empty() {
        return Err(Error::NotFound("PVsyst no está en ejecución".into()));
    }
    pids.into_iter()
        .find_map(|pid| fallo_de(pid).map(|f| (pid, f)))
        .ok_or_else(|| {
            Error::Invalid(
                "PVsyst no muestra un diálogo de fallo (madExcept o el aviso de inestabilidad): no hay nada que reiniciar. Usa cerrar o lanzar".into(),
            )
        })
}

/// Restarts PVsyst from its failure screen: the madExcept dialog's own
/// restart button, or "No" on the instability warning followed by a launch.
pub(crate) fn reiniciar(paths: &crate::config::Paths, args: &Value) -> Result<Value> {
    let (pid, fallo) = proceso_en_fallo(args)?;
    let salida = salir_del_fallo(&fallo)?;
    if !esperar_salida(pid) {
        return Err(Error::Gui(format!(
            "PVsyst (pid {pid}) no terminó en {} s tras pulsar el botón del diálogo",
            ESPERA_SALIDA.as_secs()
        )));
    }
    let reaparicion = if salida.pvsyst_se_reinicia {
        nueva_instancia(pid)
    } else {
        Reaparicion::Ninguna
    };
    let pid_nuevo = match reaparicion {
        Reaparicion::Lista(p) => p,
        Reaparicion::SinInicio(p) => {
            return Err(Error::Gui(format!(
                "PVsyst se reinició (pid {p}) pero su pantalla de inicio no apareció en {} s",
                ESPERA_ARRANQUE.as_secs()
            )))
        }
        Reaparicion::Ninguna => arrancar_de_nuevo(paths)?,
    };
    Ok(json!({ "accion": "reiniciar", "via": salida.via, "pid_anterior": pid, "pid": pid_nuevo }))
}

fn arrancar_de_nuevo(paths: &crate::config::Paths) -> Result<u32> {
    let lanzado = lanzar(paths, &json!({}))?;
    lanzado
        .get("pid")
        .and_then(Value::as_u64)
        .and_then(|p| u32::try_from(p).ok())
        .ok_or_else(|| Error::Gui("PVsyst no devolvió pid al lanzarse".into()))
}
