//! What an agent asks of the 3D editor, validated without PVsyst: objects by
//! name and number, never bytes. PVsyst writes its own scene files; the MCP
//! only describes the scene, checks it and, later, compares what PVsyst saved.

mod catalogo;
mod colision;
mod formas;
mod informe;
mod lectura;
mod problema;
mod verificacion;

pub(crate) use informe::{describir_escena, nombre_valido};
pub(crate) use lectura::{leer_objetos, ObjetoPedido, MAX_OBJETOS};
pub(crate) use verificacion::comparar;

#[cfg(test)]
#[path = "../../../tests/unit/escena_3d/pedido.rs"]
mod tests;
