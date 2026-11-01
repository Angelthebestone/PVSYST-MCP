//! Puts one value into one live control: types it, picks it from a list or clicks it.

use super::campos_vivos::{valor_de, DESCONOCIDO};
use super::dominio::{CampoEsquema, TipoValor};
use crate::adapters::gui::conduccion::Formulario;
use crate::adapters::gui::ffi::{self, Hwnd};
use crate::error::{Error, Result};

pub(super) struct EscrituraDestino<'a> {
    pub(super) formulario: &'a Formulario,
    pub(super) campo: &'a CampoEsquema,
    pub(super) control: Hwnd,
}

/// What a check box or option needs so it reads as asked.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum Marca {
    /// It already reads as asked: a click would flip it.
    YaEsta,
    Pulsar,
    /// Its state could not be read, so a click could flip it the wrong way.
    SinLectura,
}

pub(super) fn decidir_marca(actual: &str, pedido: &str) -> Marca {
    if actual == pedido {
        Marca::YaEsta
    } else if actual == DESCONOCIDO {
        Marca::SinLectura
    } else {
        Marca::Pulsar
    }
}

/// Chooses an option or toggles a check box so it reads `texto`; an option that is on
/// stays on until another of its group is chosen (`planificar` refuses to ask it off).
/// A state that cannot be read is read once more and, if still unknown, nothing is clicked.
fn marcar(destino: &EscrituraDestino<'_>, texto: &str) -> Result<()> {
    let clase = destino.campo.clase;
    let mut marca = decidir_marca(&valor_de(destino.control, clase), texto);
    if marca == Marca::SinLectura {
        marca = decidir_marca(&valor_de(destino.control, clase), texto);
    }
    match marca {
        Marca::YaEsta => Ok(()),
        Marca::Pulsar => {
            destino.formulario.marcar_en(destino.control);
            Ok(())
        }
        Marca::SinLectura => Err(Error::Gui(format!(
            "No se pudo leer el estado de '{}': no se pulsa para no invertirlo",
            destino.campo.clave
        ))),
    }
}

pub(super) fn escribir(destino: &EscrituraDestino<'_>, texto: &str) -> Result<()> {
    if destino.campo.tipo == TipoValor::Casilla {
        return marcar(destino, texto);
    }
    if destino.campo.tipo == TipoValor::Lista {
        if ffi::elegir_en_combo(destino.control, texto)? {
            return Ok(());
        }
        return Err(Error::Invalid(format!(
            "La lista '{}' no ofrece '{texto}'",
            destino.campo.clave
        )));
    }
    destino.formulario.escribir_en(destino.control, texto)?;
    Ok(())
}

#[cfg(test)]
#[path = "../../tests/unit/vista_agente/escritura.rs"]
mod tests;
