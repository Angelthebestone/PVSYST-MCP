//! A PVsyst GUI session for the MCP's own steps: start PVsyst if needed, make
//! sure the user has nothing open, answer PVsyst's message boxes and leave the
//! home screen as it was found, also when a step fails.

use super::conduccion::{esperar, formularios, Formulario, ESPERA_FORMULARIO};
use super::registro_pvsyst::{self, Marca};
use super::ventanas::{class_of, find_pids};
use super::{acciones, ffi};
use crate::config::Paths;
use crate::error::{Error, Result};
use std::time::Duration;

const EJECUTABLE: &str = "PVsyst.exe";
const INICIO: &str = "THomeForm";
const AVISO_PRUEBA: &str = "TDEMOForm";
const MENSAJE: &str = "TMessageForm";
/// Windows that are not forms the user works in.
const AUXILIARES: [&str; 4] = ["TApplication", "THintWindow", INICIO, AVISO_PRUEBA];
/// PVsyst needs ~16 s to start.
const ARRANQUE: Duration = Duration::from_secs(60);
/// Rounds of closing when cleaning up after a failed step.
const MAX_CIERRES: usize = 12;
/// Buttons that leave a form without keeping its changes, in preference order.
const SALIDAS: [&str; 4] = ["Cancelar", "Cerrar", "Salida", "No"];

/// PVsyst on its home screen, ready for one MCP step.
pub(crate) struct Sesion {
    pub(crate) pid: u32,
    pub(crate) inicio: Formulario,
    marca: Option<Marca>,
}

fn pid_activo() -> Option<u32> {
    find_pids(EJECUTABLE).into_iter().next()
}

fn arrancar(paths: &Paths) -> Result<u32> {
    if let Some(pid) = pid_activo() {
        return Ok(pid);
    }
    super::proceso::lanzar(paths, &serde_json::json!({}))?;
    pid_activo().ok_or_else(|| Error::Gui("PVsyst no arrancó".into()))
}

/// Forms open in PVsyst other than the home screen and helpers.
fn formularios_de_usuario(pid: u32) -> Vec<String> {
    formularios(pid)
        .into_iter()
        .map(class_of)
        .filter(|c| !AUXILIARES.contains(&c.as_str()))
        .collect()
}

impl Sesion {
    /// Starts (or finds) PVsyst and checks it is idle on its home screen.
    pub(crate) fn abrir(paths: &Paths) -> Result<Self> {
        acciones::exigir_escritorio()?;
        let pid = arrancar(paths)?;
        let inicio = esperar(pid, INICIO, ARRANQUE)?;
        if let Ok(aviso) = esperar(pid, AVISO_PRUEBA, Duration::ZERO) {
            aviso.pulsar("Cerrar")?;
        }
        let abiertos = formularios_de_usuario(pid);
        if !abiertos.is_empty() {
            return Err(Error::Invalid(format!(
                "PVsyst tiene ventanas abiertas ({}): ciérralas y vuelve a la pantalla de inicio",
                abiertos.join(", ")
            )));
        }
        Ok(Self {
            pid,
            inicio,
            marca: registro_pvsyst::marcar(),
        })
    }

    pub(crate) fn esperar(&self, clase: &str) -> Result<Formulario> {
        esperar(self.pid, clase, ESPERA_FORMULARIO)
    }

    /// Accepts every PVsyst message box currently shown. Informational warnings
    /// have only "OK"; confirmation and error dialogs use the other captions.
    /// Their text is in [`Sesion::mensajes`].
    pub(crate) fn aceptar_mensajes(&self) -> usize {
        const BOTONES: [&str; 5] = ["OK", "Sí", "Aceptar", "Continuar", "Cerrar"];
        let mut n = 0;
        while let Ok(m) = esperar(self.pid, MENSAJE, Duration::from_millis(600)) {
            let mut cerrado = false;
            for boton in BOTONES {
                if m.pulsar(boton).is_ok() {
                    cerrado = true;
                    break;
                }
            }
            if !cerrado {
                ffi::post_message(m.h, ffi::WM_CLOSE, 0, 0);
            }
            n += 1;
        }
        n
    }

    /// Messages and warnings PVsyst logged since the session started.
    pub(crate) fn mensajes(&self) -> Vec<String> {
        self.marca
            .as_ref()
            .map(registro_pvsyst::mensajes_desde)
            .unwrap_or_default()
    }

    /// Closes every form back to the home screen without saving.
    pub(crate) fn volver_al_inicio(&self) {
        for _ in 0..MAX_CIERRES {
            let Some(h) = formularios(self.pid)
                .into_iter()
                .find(|h| !AUXILIARES.contains(&class_of(*h).as_str()))
            else {
                return;
            };
            let f = Formulario::de(h);
            if !SALIDAS.iter().any(|b| f.pulsar(b).is_ok()) {
                ffi::post_message(h, ffi::WM_CLOSE, 0, 0);
            }
            std::thread::sleep(Duration::from_millis(500));
        }
    }

    /// Runs `paso`; whatever happens, PVsyst ends on its home screen.
    pub(crate) fn ejecutar<T>(&self, paso: impl FnOnce(&Self) -> Result<T>) -> Result<T> {
        let r = paso(self);
        self.volver_al_inicio();
        r.map_err(|e| match self.mensajes() {
            m if m.is_empty() => e,
            m => Error::Gui(format!("{e}. PVsyst dijo: {}", m.join(" | "))),
        })
    }
}
