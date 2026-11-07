//! Project lifecycle: creation, edition, variant management, and sizing.

pub(crate) mod componentes_variante;
pub(crate) mod creacion;
pub(crate) mod dimensionamiento;
pub(crate) mod edicion;
pub(crate) mod plantilla;
mod potencia_modulo;
pub(crate) mod sitio;
mod transacciones;
pub(crate) mod variantes;

pub(crate) use creacion::{crear_proyecto, eliminar_proyecto, renombrar_proyecto};
pub(crate) use dimensionamiento::{
    dimensionar_variante, validar_parametros as validar_dimensionado,
};
pub(crate) use edicion::editar_variante;
pub(crate) use potencia_modulo::pnom_from_name;
pub(crate) use variantes::{
    abrir_variante, crear_variante, guardar_variante, respuesta_de_edicion,
};
