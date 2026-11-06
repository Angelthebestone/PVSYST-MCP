//! Variant losses: validated percentages, object changes and the
//! `proyecto accion:perdidas` use case.

mod aplicacion;
mod dominio;

pub(crate) use dominio::validar_parametros;

pub(crate) use aplicacion::aplicar_perdidas;
