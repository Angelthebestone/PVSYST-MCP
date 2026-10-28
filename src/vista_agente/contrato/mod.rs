//! Contract of the `vista` block every tool answer carries: state, summary,
//! keyed values, warnings, executable next steps and what the detail level cut.
//! Pure: no Win32, no I/O. The server layer applies it to each answer.

mod campos;
pub(crate) mod catalogo;
mod construccion;
mod errores;
mod pendiente;
mod tipos;

pub(crate) use construccion::{construir, Contexto};
pub(crate) use errores::desde_error;
pub(crate) use tipos::Vista;

#[cfg(test)]
#[path = "../../../tests/unit/vista_agente/contrato/mod.rs"]
mod tests;
