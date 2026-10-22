//! Registry of the forms the agent view knows.

mod paneles;

use super::dominio::Entidad;
use crate::error::{Error, Result};

static ENTIDADES: &[&Entidad] = &[&paneles::PANELES];

pub(crate) fn todas() -> &'static [&'static Entidad] {
    ENTIDADES
}

pub(crate) fn entidad(clave: &str) -> Result<&'static Entidad> {
    todas()
        .iter()
        .copied()
        .find(|e| e.clave == clave)
        .ok_or_else(|| {
            let validas: Vec<&str> = todas().iter().map(|e| e.clave).collect();
            Error::Invalid(format!("Entidad '{clave}' desconocida: usa {validas:?}"))
        })
}

/// The entity whose window class is `clase`, when there is one.
pub(crate) fn de_formulario(clase: &str) -> Option<&'static Entidad> {
    todas().iter().copied().find(|e| e.formulario == clase)
}
