//! Drives PVsyst forms the way a user does: buttons and options by caption,
//! unlabeled fields by their position on the form, tab pages, combo boxes and
//! the message boxes PVsyst raises. Everything the form computes and saves is
//! done by PVsyst itself.
//!
//! Positions are px at 96 DPI from the corner of the window rectangle, the same
//! ones `gui_observar que:vista` publishes (see [`super::posicion`]).

use super::ffi::{self, Hwnd};
use super::posicion;
use super::sondeo::{esperar_mientras, sondear_hasta};
use super::ventanas::{class_of, enum_windows_de, pid_of, rect_of, text_of, visible};
use crate::error::{Error, Result};
use crate::idioma::{Idioma, Texto};
use std::time::Duration;

const SONDEO: Duration = Duration::from_millis(200);
/// Lets the VCL form react (recalculate, open the next form) after an input.
const PAUSA_ACCION: Duration = Duration::from_millis(400);
/// Default time a form gets to appear.
pub(crate) const ESPERA_FORMULARIO: Duration = Duration::from_secs(15);

/// One field without a caption, located by its centre on the form.
#[derive(Debug, Clone, Copy)]
pub(crate) struct Campo {
    pub(crate) clase: &'static str,
    pub(crate) x: i32,
    pub(crate) y: i32,
}

/// A visible top-level PVsyst form.
#[derive(Debug, Clone, Copy)]
pub(crate) struct Formulario {
    pub(crate) h: Hwnd,
    dpi: u32,
    /// Client-facing language of the messages this form raises.
    idioma: Idioma,
}

/// Visible top-level windows of process `pid`, front-most first.
pub(crate) fn formularios(pid: u32) -> Vec<Hwnd> {
    enum_windows_de(pid)
        .into_iter()
        .filter(|h| visible(*h))
        .collect()
}

/// Waits until a visible form of class `clase` exists in `pid`.
pub(crate) fn esperar(pid: u32, clase: &str, espera: Duration) -> Result<Formulario> {
    sondear_hasta(espera, SONDEO, || {
        formularios(pid).into_iter().find(|h| class_of(*h) == clase)
    })
    .map(Formulario::de)
    .ok_or_else(|| {
        Error::Gui(format!(
            "PVsyst did not open '{clase}' in {} s",
            espera.as_secs()
        ))
    })
}

/// Waits until `h` is gone or hidden; `false` on timeout.
pub(crate) fn esperar_cierre(h: Hwnd, espera: Duration) -> bool {
    esperar_mientras(espera, SONDEO, || ffi::es_ventana(h) && visible(h))
}

impl Formulario {
    /// A form for a caller that carries no `Idioma` (`vista_agente`): the
    /// environment is the same source `Paths` reads.
    pub(crate) fn de(h: Hwnd) -> Self {
        Self::con_idioma(h, Idioma::de_entorno())
    }

    /// A form driven by a call that knows the client's language.
    pub(crate) fn con_idioma(h: Hwnd, idioma: Idioma) -> Self {
        Self {
            h,
            dpi: ffi::dpi_de(h),
            idioma,
        }
    }

    pub(crate) fn clase(&self) -> String {
        class_of(self.h)
    }

    pub(crate) fn titulo(&self) -> String {
        text_of(self.h)
    }

    pub(crate) fn pid(&self) -> u32 {
        pid_of(self.h)
    }

    fn escalar(&self, v: i32) -> i32 {
        ffi::escalar_a_dpi(v, self.dpi)
    }

    fn hijos_visibles(&self) -> impl Iterator<Item = Hwnd> {
        ffi::enum_children(self.h)
            .into_iter()
            .filter(|c| visible(*c))
    }

    /// The visible child whose caption is exactly `rotulo`. O(n) in the children.
    pub(crate) fn hijo(&self, rotulo: &str) -> Result<Hwnd> {
        self.hijos_visibles()
            .find(|c| text_of(*c) == rotulo)
            .ok_or_else(|| {
                Error::Gui(format!(
                    "'{rotulo}' is not in '{}' of PVsyst",
                    self.titulo()
                ))
            })
    }

    /// Whether the captioned control exists and is enabled.
    pub(crate) fn habilitado(&self, rotulo: &str) -> bool {
        self.hijo(rotulo)
            .is_ok_and(|c| ffi::estado_ventana(c).habilitada)
    }

    /// Clicks the button captioned `rotulo` with the mechanism its class needs
    /// (`BM_CLICK` for standard buttons, mouse messages for `TPngBitBtn`).
    pub(crate) fn pulsar(&self, rotulo: &str) -> Result<()> {
        let b = self.hijo(rotulo)?;
        super::acciones::clic_por_mensajes(b)?;
        std::thread::sleep(PAUSA_ACCION);
        Ok(())
    }

    /// Clicks the radio button or check box captioned `rotulo` with the mouse,
    /// so the form runs the same handlers as for a user.
    pub(crate) fn elegir(&self, rotulo: &str) -> Result<()> {
        let c = self.hijo(rotulo)?;
        self.clic_en_control(c);
        Ok(())
    }

    /// The control of `campo.clase` at the position `vista` publishes: px at 96
    /// DPI from the corner of the window rectangle, not of its client area.
    pub(crate) fn control(&self, campo: Campo) -> Result<Hwnd> {
        let candidatos = self
            .hijos_visibles()
            .filter(|c| class_of(*c) == campo.clase)
            .map(|c| (c, rect_of(c)));
        posicion::control_en(
            (rect_of(self.h), ffi::escala_de_captura(self.h)),
            (campo.x, campo.y),
            candidatos,
        )
        .ok_or_else(|| {
            let ausente = Texto {
                es: "No hay {clase} en ({x}, {y}) de '{titulo}': ¿cambió la versión de PVsyst?",
                en: "No {clase} at ({x}, {y}) of '{titulo}': did the PVsyst version change?",
            };
            Error::Gui(ausente.texto_con(
                self.idioma,
                &[
                    ("clase", campo.clase),
                    ("x", &campo.x.to_string()),
                    ("y", &campo.y.to_string()),
                    ("titulo", &self.titulo()),
                ],
            ))
        })
    }

    /// Clicks control `c` of this form with the mouse (check boxes and options).
    pub(crate) fn marcar_en(&self, c: Hwnd) {
        self.clic_en_control(c);
    }

    fn clic_en_control(&self, c: Hwnd) {
        let r = rect_of(c);
        ffi::traer_al_frente(self.h);
        ffi::clic_fisico_en((r.left + r.right) / 2, (r.top + r.bottom) / 2);
        std::thread::sleep(PAUSA_ACCION);
    }

    /// The only visible control of class `clase` on the form.
    pub(crate) fn unico(&self, clase: &str) -> Result<Hwnd> {
        let mut it = self.hijos_visibles().filter(|c| class_of(*c) == clase);
        match (it.next(), it.next()) {
            (Some(c), None) => Ok(c),
            _ => Err(Error::Gui(format!(
                "'{}' does not have a single {clase}",
                self.titulo()
            ))),
        }
    }

    /// Types into the form's only control of class `clase` (e.g. a search box).
    pub(crate) fn escribir_unico(&self, clase: &str, texto: &str) -> Result<()> {
        let c = self.unico(clase)?;
        self.clic_en_control(c);
        ffi::teclear_en(c, texto)?;
        std::thread::sleep(PAUSA_ACCION);
        Ok(())
    }

    /// Clicks at `(dx, dy)` (px at 96 DPI) from the top-left corner of the
    /// form's only control of class `clase` (e.g. a cell of a list).
    pub(crate) fn clic_en_unico(&self, clase: &str, (dx, dy): (i32, i32)) -> Result<()> {
        let r = rect_of(self.unico(clase)?);
        ffi::traer_al_frente(self.h);
        ffi::clic_fisico_en(r.left + self.escalar(dx), r.top + self.escalar(dy));
        std::thread::sleep(PAUSA_ACCION);
        Ok(())
    }

    /// Types `texto` into the field and commits it with Tab.
    pub(crate) fn escribir(&self, campo: Campo, texto: &str) -> Result<()> {
        let c = self.control(campo)?;
        self.escribir_en(c, texto)?;
        Ok(())
    }

    /// Types `texto` into control `c` of this form and commits it with Tab.
    pub(crate) fn escribir_en(&self, c: Hwnd, texto: &str) -> Result<()> {
        self.clic_en_control(c);
        ffi::teclear_en(c, texto)?;
        ffi::pulsar_tabulador();
        std::thread::sleep(PAUSA_ACCION);
        Ok(())
    }

    /// Picks the first item of the combo at `campo` that starts with `prefijo`.
    pub(crate) fn seleccionar(&self, campo: Campo, prefijo: &str) -> Result<()> {
        let c = self.control(campo)?;
        if !ffi::elegir_en_combo(c, prefijo)? {
            return Err(Error::Invalid(format!(
                "PVsyst offers no '{prefijo}' in that list of '{}' (items are matched by prefix, not by substring)",
                self.titulo()
            )));
        }
        std::thread::sleep(PAUSA_ACCION);
        Ok(())
    }

    /// Shows tab page `indice` of the form's (first) page control. PVsyst may veto
    /// the change while the current page has invalid fields, so the selected tab
    /// is read back and a refusal is an error.
    pub(crate) fn pagina(&self, indice: usize) -> Result<()> {
        let pc = self
            .hijos_visibles()
            .find(|c| class_of(*c) == ffi::CLASE_PESTANAS)
            .ok_or_else(|| Error::Gui(format!("'{}' has no tab pages", self.titulo())))?;
        let total = ffi::paginas_de(pc).unwrap_or(usize::MAX);
        ffi::exigir_indice_pagina(indice, total, &self.titulo())?;
        ffi::mostrar_pagina(pc, indice)?;
        std::thread::sleep(PAUSA_ACCION);
        ffi::exigir_cambio_de_pestana(indice, ffi::pagina_seleccionada(pc))
    }
}
