//! Grid-injection limit: parameters, validated changes and the
//! `proyecto accion:limite_inyeccion` use case.

mod aplicacion;
mod dominio;

pub(crate) use dominio::validar_parametros;

pub(crate) use aplicacion::definir_limite;
