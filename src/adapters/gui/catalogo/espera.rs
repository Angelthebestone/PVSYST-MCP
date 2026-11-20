//! Observación de ventanas nuevas y abiertas como respuesta a acciones de la GUI.

use super::dominio::Funcion;
use crate::adapters::gui::ffi::Hwnd;
use crate::adapters::gui::ventanas::{class_of, window_json};
use crate::error::Result;
use serde_json::Value;
use std::time::{Duration, Instant};

const SONDEO: Duration = Duration::from_millis(100);

/// The classes that count as the window a catalogued function opens: the one it
/// announces (`Funcion::abre`) and any alternative measured on another kind of
/// project.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) struct VentanaEsperada {
    principal: &'static str,
    alternativa: Option<&'static str>,
}

impl VentanaEsperada {
    fn incluye(self, clase: &str) -> bool {
        self.principal == clase || self.alternativa == Some(clase)
    }

    /// The accepted classes, joined for the answer that names them all when
    /// none of them opened (`TDomesticUseForm / TUserNeedsForm`).
    pub(super) fn listadas(self) -> String {
        match self.alternativa {
            Some(alternativa) => format!("{} / {alternativa}", self.principal),
            None => self.principal.to_string(),
        }
    }
}

/// The windows `funcion` may show; `None` when none was ever observed.
pub(super) fn esperadas(funcion: &Funcion) -> Option<VentanaEsperada> {
    Some(VentanaEsperada {
        principal: funcion.abre?,
        alternativa: funcion.abre_tambien,
    })
}

/// Ventanas aparecidas desde `antes`.
pub(super) fn ventanas_nuevas(ahora: &[Hwnd], antes: &[Hwnd]) -> Vec<Hwnd> {
    ahora
        .iter()
        .copied()
        .filter(|h| !antes.contains(h))
        .collect()
}

/// Ventanas que cuentan como la esperada entre `ventanas`.
fn de_las_aceptadas(ventanas: &[Hwnd], esperada: VentanaEsperada) -> Vec<Hwnd> {
    ventanas
        .iter()
        .copied()
        .filter(|h| esperada.incluye(&class_of(*h)))
        .collect()
}

/// Resultado de observar la aparición de ventanas tras una invocación.
pub(super) struct Resultado {
    pub(super) nuevas: Vec<Hwnd>,
    /// Ventanas de la clase esperada que ya estaban abiertas antes.
    pub(super) ya_abiertas: Vec<Hwnd>,
    /// Llegó la ventana esperada (o alguna nueva, si no se esperaba una clase).
    pub(super) llego: bool,
}

impl Resultado {
    pub(super) fn vacio() -> Self {
        Self {
            nuevas: Vec::new(),
            ya_abiertas: Vec::new(),
            llego: false,
        }
    }

    /// The accepted windows of which none showed up, nor was already open: the
    /// action did not do what the catalogue says it does.
    pub(super) fn ausente(&self, esperada: Option<VentanaEsperada>) -> Option<VentanaEsperada> {
        esperada.filter(|_| !self.llego && self.ya_abiertas.is_empty())
    }

    pub(super) fn nuevas_json(&self) -> Vec<Value> {
        self.nuevas.iter().map(|h| window_json(*h)).collect()
    }

    pub(super) fn primera_nueva(&self) -> Option<Hwnd> {
        self.nuevas.first().copied()
    }

    pub(super) fn ya_abiertas_json(&self) -> Vec<Value> {
        self.ya_abiertas.iter().map(|h| window_json(*h)).collect()
    }
}

fn calcular_ya_abiertas(
    antes: &[Hwnd],
    ahora: &[Hwnd],
    esperada: Option<VentanaEsperada>,
) -> Vec<Hwnd> {
    let Some(esperada) = esperada else {
        return Vec::new();
    };
    de_las_aceptadas(antes, esperada)
        .into_iter()
        .filter(|h| ahora.contains(h))
        .collect()
}

#[derive(Clone, Copy)]
pub(super) struct SolicitudEspera<'a> {
    pub(super) antes: &'a [Hwnd],
    pub(super) esperada: Option<VentanaEsperada>,
    pub(super) limite: Duration,
}

/// Espera a que aparezca una ventana que no estaba antes (de alguna de las clases aceptadas si se indicaron).
pub(super) fn esperar_nuevas<F>(
    solicitud: SolicitudEspera<'_>,
    mut consultar_visibles: F,
) -> Result<Resultado>
where
    F: FnMut() -> Result<Vec<Hwnd>>,
{
    let fin = Instant::now() + solicitud.limite;
    let antes = solicitud.antes;
    let esperada = solicitud.esperada;
    loop {
        // A failed enumeration is an error, not "no window appeared".
        let ahora = consultar_visibles()?;
        let nuevas = ventanas_nuevas(&ahora, antes);
        let llegada = match esperada {
            Some(esperada) => !de_las_aceptadas(&nuevas, esperada).is_empty(),
            None => !nuevas.is_empty(),
        };
        if llegada || Instant::now() >= fin {
            let ya_abiertas = calcular_ya_abiertas(antes, &ahora, esperada);
            return Ok(Resultado {
                nuevas,
                ya_abiertas,
                llego: llegada,
            });
        }
        std::thread::sleep(SONDEO);
    }
}
