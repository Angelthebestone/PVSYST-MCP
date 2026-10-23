//! Drives PVsyst forms the way a user does: buttons and options by caption,
//! unlabeled fields by their position on the form, tab pages, combo boxes and
//! the message boxes PVsyst raises. Everything the form computes and saves is
//! done by PVsyst itself.
//!
//! Positions are client coordinates measured at 96 DPI on PVsyst 8.1.7 and are
//! scaled to the window's DPI.

use super::ffi::{self, Hwnd};
use super::ventanas::{class_of, enum_windows_de, pid_of, rect_of, text_of, visible};
use crate::error::{Error, Result};
use std::time::{Duration, Instant};

const SONDEO: Duration = Duration::from_millis(200);
/// Lets the VCL form react (recalculate, open the next form) after an input.
const PAUSA_ACCION: Duration = Duration::from_millis(400);
/// How far (px at 96 DPI) a measured position may be from a control's edge.
const TOLERANCIA_PX: i32 = 6;
/// Default time a form gets to appear.
pub(crate) const ESPERA_FORMULARIO: Duration = Duration::from_secs(15);

/// One field without a caption, located by its centre on the form.
#[derive(Debug, Clone, Copy)]
pub(crate) struct Campo {
    pub(crate) clase: &'static str,
    pub(crate) x: i32,
    pub(crate) y: i32,
}

impl Campo {
    pub(crate) const fn edit(x: i32, y: i32) -> Self {
        Self {
            clase: "TEdit",
            x,
            y,
        }
    }
    pub(crate) const fn combo(x: i32, y: i32) -> Self {
        Self {
            clase: "TComboBox",
            x,
            y,
        }
    }
    pub(crate) const fn opcion(x: i32, y: i32) -> Self {
        Self {
            clase: "TRadioButton",
            x,
            y,
        }
    }
}

/// A visible top-level PVsyst form.
#[derive(Debug, Clone, Copy)]
pub(crate) struct Formulario {
    pub(crate) h: Hwnd,
    dpi: u32,
}

/// Visible top-level windows of process `pid`, front-most first.
pub(crate) fn formularios(pid: u32) -> Vec<Hwnd> {
    enum_windows_de(pid)
        .into_iter()
        .filter(|h| visible(*h))
        .collect()
}

pub(crate) fn sondear_hasta<T, F>(plazo: Duration, intervalo: Duration, mut sondeo: F) -> Option<T>
where
    F: FnMut() -> Option<T>,
{
    let fin = Instant::now() + plazo;
    loop {
        if let Some(valor) = sondeo() {
            return Some(valor);
        }
        if Instant::now() >= fin {
            return None;
        }
        std::thread::sleep(intervalo);
    }
}

pub(crate) fn esperar_mientras<F>(plazo: Duration, intervalo: Duration, mut condicion: F) -> bool
where
    F: FnMut() -> bool,
{
    sondear_hasta(plazo, intervalo, || (!condicion()).then_some(())).is_some()
}

/// Waits until a visible form of class `clase` exists in `pid`.
pub(crate) fn esperar(pid: u32, clase: &str, espera: Duration) -> Result<Formulario> {
    sondear_hasta(espera, SONDEO, || {
        formularios(pid).into_iter().find(|h| class_of(*h) == clase)
    })
    .map(Formulario::de)
    .ok_or_else(|| Error::Gui(format!("PVsyst no abrió '{clase}' en {} s", espera.as_secs())))
}

/// Waits until `h` is gone or hidden; `false` on timeout.
pub(crate) fn esperar_cierre(h: Hwnd, espera: Duration) -> bool {
    esperar_mientras(espera, SONDEO, || ffi::es_ventana(h) && visible(h))
}

/// Target point inside an action toolbar of a form.
#[derive(Clone, Copy, Debug)]
pub(crate) struct PuntoBarra {
    pub(crate) n: usize,
    pub(crate) ancho_min: i32,
    pub(crate) x: i32,
}

impl Formulario {
    pub(crate) fn de(h: Hwnd) -> Self {
        Self {
            h,
            dpi: ffi::dpi_de(h),
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
                    "'{rotulo}' no está en '{}' de PVsyst",
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

    /// The control of `campo.clase` at the measured position.
    pub(crate) fn control(&self, campo: Campo) -> Result<Hwnd> {
        let (ox, oy) = ffi::origen_cliente(self.h);
        let (x, y) = (ox + self.escalar(campo.x), oy + self.escalar(campo.y));
        let tol = self.escalar(TOLERANCIA_PX);
        self.hijos_visibles()
            .filter(|c| class_of(*c) == campo.clase)
            .find(|c| {
                let r = rect_of(*c);
                r.left - tol <= x && x <= r.right + tol && r.top - tol <= y && y <= r.bottom + tol
            })
            .ok_or_else(|| {
                Error::Gui(format!(
                    "No hay {} en ({}, {}) de '{}': ¿cambió la versión de PVsyst?",
                    campo.clase,
                    campo.x,
                    campo.y,
                    self.titulo()
                ))
            })
    }

    fn clic_en_control(&self, c: Hwnd) {
        let r = rect_of(c);
        ffi::traer_al_frente(self.h);
        ffi::clic_fisico_en((r.left + r.right) / 2, (r.top + r.bottom) / 2);
        std::thread::sleep(PAUSA_ACCION);
    }

    /// The only visible control of class `clase` on the form.
    fn unico(&self, clase: &str) -> Result<Hwnd> {
        let mut it = self.hijos_visibles().filter(|c| class_of(*c) == clase);
        match (it.next(), it.next()) {
            (Some(c), None) => Ok(c),
            _ => Err(Error::Gui(format!(
                "'{}' no tiene un único {clase}",
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
    /// form's only control of class `clase` (e.g. the first row of a list).
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

    pub(crate) fn leer(&self, campo: Campo) -> Result<String> {
        self.control(campo).map(ffi::texto_control)
    }

    /// Clicks the unlabeled option (radio button) at `campo`.
    pub(crate) fn marcar(&self, campo: Campo) -> Result<()> {
        let c = self.control(campo)?;
        self.clic_en_control(c);
        Ok(())
    }

    /// Picks the first item of the combo at `campo` that starts with `prefijo`.
    pub(crate) fn seleccionar(&self, campo: Campo, prefijo: &str) -> Result<()> {
        let c = self.control(campo)?;
        if !ffi::elegir_en_combo(c, prefijo)? {
            return Err(Error::Invalid(format!(
                "PVsyst no ofrece '{prefijo}' en esa lista de '{}'",
                self.titulo()
            )));
        }
        std::thread::sleep(PAUSA_ACCION);
        Ok(())
    }

    /// Shows tab page `indice` of the form's (first) page control.
    pub(crate) fn pagina(&self, indice: usize) -> Result<()> {
        let pc = self
            .hijos_visibles()
            .find(|c| class_of(*c) == "TPageControl")
            .ok_or_else(|| Error::Gui(format!("'{}' no tiene pestañas", self.titulo())))?;
        ffi::mostrar_pagina(pc, indice)?;
        std::thread::sleep(PAUSA_ACCION);
        Ok(())
    }

    /// Clicks a point of a toolbar (`TActionToolBar`), the only way to reach
    /// its actions: the `n`-th toolbar wider than `ancho_min`, top to bottom.
    pub(crate) fn clic_barra(&self, punto: PuntoBarra) -> Result<()> {
        let mut barras: Vec<ffi::Rect> = self
            .hijos_visibles()
            .filter(|c| class_of(*c) == "TActionToolBar")
            .map(rect_of)
            .filter(|r| r.right - r.left >= self.escalar(punto.ancho_min))
            .collect();
        barras.sort_by_key(|r| r.top);
        let r = barras
            .get(punto.n)
            .ok_or_else(|| Error::Gui(format!("'{}' no tiene esa barra", self.titulo())))?;
        ffi::traer_al_frente(self.h);
        ffi::clic_fisico_en(r.left + self.escalar(punto.x), (r.top + r.bottom) / 2);
        std::thread::sleep(PAUSA_ACCION);
        Ok(())
    }
}
