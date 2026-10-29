//! What an agent asks of the 3D editor, validated without PVsyst: objects by
//! name and number, never bytes. PVsyst writes its own scene files; the MCP
//! only describes the scene, checks it and, later, compares what PVsyst saved.

mod catalogo;
mod colision;
mod formas;
mod informe;
mod lectura;
mod problema;

pub(crate) use informe::describir_escena;
pub(crate) use lectura::MAX_OBJETOS;

#[cfg(test)]
#[path = "../../../tests/unit/escena_3d/pedido.rs"]
mod tests;
