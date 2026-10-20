//! Generic, workspace-guarded file operations.
//!
//! These are the "escape hatch" primitives: with them the MCP can reach *any*
//! file PVsyst uses (projects, components, sites, weather, models, shading
//! scenes, batch files, ...). Writes are confined to the workspace unless the
//! caller explicitly opts out.

mod base64;
mod escritura;
mod lectura;
mod listado;
mod nombres;
mod recorrido;
mod rutas;
#[cfg(test)]
#[path = "../../tests/unit/workspace/tests.rs"]
mod tests;

pub(crate) use crate::config::Alcance;
pub(crate) use escritura::escribir_archivo;
pub(crate) use lectura::leer_archivo;
pub(crate) use listado::{listar_archivos, ResumenRecorrido};
pub(crate) use nombres::validar_nombre;
pub(crate) use recorrido::walk_ext;
pub(crate) use rutas::{mover_a_papelera, resolver_ruta};
