//! The catalogue data, one file per area of the interface.

mod ayuda;
mod dialogos_inicio;
mod dialogos_variante;
mod escena_3d;
mod inicio;
mod tipos_de_sistema;
mod utilidades;
mod variante;
mod ventana_proyecto;

use super::dominio::Funcion;

/// Every catalogued function, in interface order.
pub(crate) fn funciones() -> impl Iterator<Item = &'static Funcion> {
    [
        inicio::FUNCIONES,
        ayuda::FUNCIONES,
        utilidades::FUNCIONES,
        variante::FUNCIONES,
        dialogos_variante::FUNCIONES,
        dialogos_inicio::FUNCIONES,
        tipos_de_sistema::FUNCIONES,
        escena_3d::FUNCIONES,
        ventana_proyecto::FUNCIONES,
    ]
    .into_iter()
    .flatten()
}
