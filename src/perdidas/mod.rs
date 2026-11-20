//! Variant losses: validated percentages, object changes and the
//! `proyecto accion:perdidas` use case.

mod aplicacion;
mod dominio;
mod flags;

pub(crate) use dominio::validar_parametros;

pub(crate) use aplicacion::aplicar_perdidas;
