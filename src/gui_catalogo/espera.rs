//! Observación de ventanas nuevas y abiertas como respuesta a acciones de la GUI.

use crate::adapters::gui::ffi::Hwnd;
use crate::adapters::gui::ventanas::{class_of, window_json};
use crate::error::Result;
use serde_json::Value;
use std::time::{Duration, Instant};

const SONDEO: Duration = Duration::from_millis(100);

/// Ventanas aparecidas desde `antes`.
pub(super) fn ventanas_nuevas(ahora: &[Hwnd], antes: &[Hwnd]) -> Vec<Hwnd> {
    ahora
        .iter()
        .copied()
        .filter(|h| !antes.contains(h))
        .collect()
}

/// Ventanas de clase `clase` entre `ventanas`.
pub(super) fn de_clase(ventanas: &[Hwnd], clase: &str) -> Vec<Hwnd> {
    ventanas
        .iter()
        .copied()
        .filter(|h| class_of(*h) == clase)
        .collect()
}

/// Resultado de observar la aparición de ventanas tras una invocación.
pub(super) struct Resultado {
    pub(super) nuevas: Vec<Hwnd>,
    /// Ventanas de la clase esperada que ya estaban abiertas antes.
    pub(super) ya_abiertas: Vec<Hwnd>,
}

impl Resultado {
    pub(super) fn vacio() -> Self {
        Self {
            nuevas: Vec::new(),
            ya_abiertas: Vec::new(),
        }
    }

    pub(super) fn nuevas_json(&self) -> Vec<Value> {
        self.nuevas.iter().map(|h| window_json(*h)).collect()
    }

    pub(super) fn ya_abiertas_json(&self) -> Vec<Value> {
        self.ya_abiertas.iter().map(|h| window_json(*h)).collect()
    }
}

fn calcular_ya_abiertas(antes: &[Hwnd], ahora: &[Hwnd], esperada: Option<&str>) -> Vec<Hwnd> {
    let Some(clase) = esperada else {
        return Vec::new();
    };
    de_clase(antes, clase)
        .into_iter()
        .filter(|h| ahora.contains(h))
        .collect()
}

#[derive(Clone, Copy)]
pub(super) struct SolicitudEspera<'a> {
    pub(super) antes: &'a [Hwnd],
    pub(super) esperada: Option<&'a str>,
    pub(super) limite: Duration,
}

/// Espera a que aparezca una ventana que no estaba antes (de clase `esperada` si se indicó).
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
        let Ok(ahora) = consultar_visibles() else {
            return Ok(Resultado::vacio());
        };
        let nuevas = ventanas_nuevas(&ahora, antes);
        let llegada = match esperada {
            Some(clase) => !de_clase(&nuevas, clase).is_empty(),
            None => !nuevas.is_empty(),
        };
        if llegada || Instant::now() >= fin {
            let ya_abiertas = calcular_ya_abiertas(antes, &ahora, esperada);
            return Ok(Resultado {
                nuevas,
                ya_abiertas,
            });
        }
        std::thread::sleep(SONDEO);
    }
}
