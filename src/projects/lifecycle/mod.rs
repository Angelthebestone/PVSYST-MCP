//! Project lifecycle: creation, edition, variant management, and sizing.

pub(crate) mod creacion;
pub(crate) mod dimensionamiento;
pub(crate) mod edicion;
mod transacciones;
pub(crate) mod variantes;

pub(crate) use creacion::{crear_proyecto, eliminar_proyecto, renombrar_proyecto};
pub(crate) use dimensionamiento::{dimensionar_variante, pnom_from_name};
pub(crate) use edicion::editar_variante;
pub(crate) use variantes::{abrir_variante, crear_variante, guardar_variante};
