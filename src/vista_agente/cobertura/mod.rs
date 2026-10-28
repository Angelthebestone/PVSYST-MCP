//! Coverage of the PVsyst windows by the agent view: each window class of the
//! observed map is registered in exactly one of four states, and windows
//! without a schema are still readable by control position.

mod basica;
mod dominio;
mod listado;
mod mapa;
mod ventanas;

pub(crate) use basica::vista_de_ventana;
pub(crate) use listado::listar_ventanas;
pub(crate) use ventanas::de_clase;

#[cfg(test)]
#[path = "../../../tests/unit/vista_agente/cobertura/mod.rs"]
mod tests;
