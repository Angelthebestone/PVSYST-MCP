//! Generic object access: read, list and edit any PVObject field of any file.

mod edicion;
mod lectura;
mod listado;
mod resolucion;
#[cfg(test)]
#[path = "../../tests/unit/objects/tests.rs"]
mod tests;

pub(crate) use edicion::editar_objeto;
pub(crate) use lectura::leer_objeto;
pub(crate) use listado::listar_objetos;
pub(crate) use resolucion::resolve_file;
