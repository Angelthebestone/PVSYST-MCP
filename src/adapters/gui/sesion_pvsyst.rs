//! A PVsyst GUI session for the MCP's own steps: start PVsyst if needed, make
//! sure the user has nothing open, answer PVsyst's message boxes and leave the
//! home screen as it was found, also when a step fails.

use super::causa_pvsyst;
use super::conduccion::{esperar, formularios, Formulario, ESPERA_FORMULARIO};
use super::registro_pvsyst::{self, Marca};
use super::rotulos::dialogos::{BOTONES_SALIDA, BOTONES_SESION, CERRAR};
use super::rotulos::Rotulo;
use super::ventanas::class_of;
use super::{acciones, ffi};
use crate::config::Paths;
use crate::error::{Error, Result};
use crate::idioma::{Idioma as IdiomaCliente, Texto};
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

/// PVsyst on its home screen, ready for one MCP step.
pub(crate) struct Sesion {
    pub(crate) pid: u32,
    pub(crate) inicio: Formulario,
    marca: Option<Marca>,
    /// The process this session started, if it did: only that one may be closed
    /// when the step fails, never a PVsyst the user already had open.
    lanzada: Option<u32>,
    idioma: IdiomaCliente,
}

fn pid_activo() -> Option<u32> {
    ffi::pids_de_ejecutable(EJECUTABLE).into_iter().next()
}

/// The running PVsyst and, when this call started it, its pid.
fn arrancar(paths: &Paths) -> Result<(u32, Option<u32>)> {
    if let Some(pid) = pid_activo() {
        return Ok((pid, None));
    }
    super::proceso::lanzar(paths, &serde_json::json!({}))?;
    let no_arranca = Texto {
        es: "PVsyst no arrancó",
        en: "PVsyst did not start",
    };
    let pid = pid_activo().ok_or_else(|| Error::Gui(no_arranca.en_idioma(paths.idioma).into()))?;
    Ok((pid, Some(pid)))
}

/// The pid to end: only a process this call started and that is still running.
/// O(n) in the running processes.
fn debe_recogerse(lanzada: Option<u32>, vivos: &[u32]) -> Option<u32> {
    let pid = lanzada?;
    vivos.contains(&pid).then_some(pid)
}

/// Ends the PVsyst this call started, so a failed step leaves no orphan GUI.
/// Returns the note to add to the error when it did.
fn recoger(lanzada: Option<u32>, idioma: IdiomaCliente) -> Option<String> {
    let pid = debe_recogerse(lanzada, &ffi::pids_de_ejecutable(EJECUTABLE))?;
    ffi::terminar_proceso(pid);
    Some(
        Texto {
            es: "PVsyst (pid {pid}) se cerró porque lo arrancó esta llamada",
            en: "PVsyst (pid {pid}) was closed because this call started it",
        }
        .texto_con(idioma, &[("pid", &pid.to_string())]),
    )
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
        let idioma = paths.idioma;
        let (pid, lanzada) = arrancar(paths)?;
        Self::en_inicio(pid, lanzada, idioma).map_err(|e| match recoger(lanzada, idioma) {
            None => e,
            Some(nota) => Error::Gui(format!("{e}. {nota}")),
        })
    }

    fn en_inicio(pid: u32, lanzada: Option<u32>, idioma: IdiomaCliente) -> Result<Self> {
        let inicio = esperar(pid, INICIO, ARRANQUE)?;
        if let Ok(aviso) = esperar(pid, AVISO_PRUEBA, Duration::ZERO) {
            aviso.pulsar(CERRAR)?;
        }
        let abiertos = formularios_de_usuario(pid);
        if !abiertos.is_empty() {
            return Err(Error::Invalid(
                Texto {
                    es: "PVsyst tiene ventanas abiertas ({ventanas}): ciérralas y vuelve a la pantalla de inicio",
                    en: "PVsyst has windows open ({ventanas}): close them and return to the home screen",
                }
                .texto_con(idioma, &[("ventanas", &abiertos.join(", "))]),
            ));
        }
        Ok(Self {
            pid,
            inicio,
            marca: registro_pvsyst::marcar(),
            lanzada,
            idioma,
        })
    }

    pub(crate) fn esperar(&self, clase: &str) -> Result<Formulario> {
        esperar(self.pid, clase, ESPERA_FORMULARIO)
    }

    /// Accepts every PVsyst message box currently shown. Informational warnings
    /// have only "OK"; confirmation and error dialogs use the other captions.
    /// Their text is in [`Sesion::mensajes`].
    pub(crate) fn aceptar_mensajes(&self) -> usize {
        let mut n = 0;
        while let Ok(m) = esperar(self.pid, MENSAJE, Duration::from_millis(600)) {
            cerrar_con_boton(&m, BOTONES_SESION);
            n += 1;
        }
        n
    }

    /// Messages and warnings PVsyst logged since the session started, without
    /// the parse notices every load emits.
    pub(crate) fn mensajes(&self) -> Vec<String> {
        let registrados = self
            .marca
            .as_ref()
            .map(registro_pvsyst::mensajes_desde)
            .unwrap_or_default();
        causa_pvsyst::filtrar(registrados)
    }

    /// Closes every form back to the home screen without saving.
    pub(crate) fn volver_al_inicio(&self) {
        let mut rondas = 0;
        while rondas < MAX_CIERRES && self.cerrar_un_formulario() {
            rondas += 1;
            std::thread::sleep(Duration::from_millis(500));
        }
    }

    fn cerrar_un_formulario(&self) -> bool {
        let Some(h) = formularios(self.pid)
            .into_iter()
            .find(|h| !AUXILIARES.contains(&class_of(*h).as_str()))
        else {
            return false;
        };
        cerrar_con_boton(&Formulario::de(h), BOTONES_SALIDA);
        true
    }

    /// Runs `paso`; whatever happens, PVsyst ends on its home screen.
    pub(crate) fn ejecutar<T>(&self, paso: impl FnOnce(&Self) -> Result<T>) -> Result<T> {
        let r = paso(self);
        self.volver_al_inicio();
        r.map_err(|e| self.error_con_causa(e))
    }

    /// `e` with PVsyst's own log turned into the cause the caller can act on,
    /// and the process this session started closed when the step failed.
    fn error_con_causa(&self, e: Error) -> Error {
        let texto = causa_pvsyst::explicar(&e.to_string(), &self.mensajes(), self.idioma);
        let texto = match recoger(self.lanzada, self.idioma) {
            None => texto,
            Some(nota) => format!("{texto}. {nota}"),
        };
        if texto == e.to_string() {
            return e;
        }
        Error::Gui(texto)
    }
}

/// Presses the first of `botones` the form has; `WM_CLOSE` when it has none.
fn cerrar_con_boton(f: &Formulario, botones: &[Rotulo<'static>]) {
    if !botones.iter().any(|b| f.pulsar(*b).is_ok()) {
        ffi::post_message(f.h, ffi::Mensaje::simple(ffi::WM_CLOSE));
    }
}

#[cfg(test)]
#[path = "../../../tests/unit/adapters/gui/sesion_pvsyst.rs"]
mod tests;
