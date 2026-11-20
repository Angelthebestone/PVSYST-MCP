//! One-axis tracker conversion: parameters, orientation changes and the
//! `proyecto accion:seguidor` use case.

mod aplicacion;
mod dominio;

pub(crate) use dominio::validar_parametros;

pub(crate) use aplicacion::convertir_a_seguidor;
