//! Hourly meteo and measured data series codec for PVsyst `.MET` and `.DAM` files.
//!
//! Provides Base62 decoding/encoding of 8760/8784-hour bodies, physical conversions,
//! boundary validation, reading paged series and atomic replacement with history snapshots.

mod aplicacion;
mod base62;
mod codificar;
mod decodificar;
mod dominio;

pub(crate) use aplicacion::{escribir_serie, leer_serie};
#[allow(unused_imports)]
pub(crate) use base62::{
    codificar_b62, codificar_linea_dia, decodificar_b62, decodificar_linea_dia,
};
#[allow(unused_imports)]
pub(crate) use codificar::codificar;
pub(crate) use decodificar::decodificar;
#[allow(unused_imports)]
pub(crate) use dominio::{
    crudo_a_fisico, fisico_a_crudo, rango_valido, validar_serie, HoraValores, MetadatosVariable,
    RangoFisico, SerieHoraria, TipoVariable, HORAS_ANO_BISIESTO, HORAS_ANO_ESTANDAR,
};

#[cfg(test)]
#[path = "../../tests/unit/serie_horaria/mod.rs"]
mod tests;
