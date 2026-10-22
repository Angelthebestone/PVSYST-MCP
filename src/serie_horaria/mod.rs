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

#[cfg(test)]
#[path = "../../tests/unit/serie_horaria/mod.rs"]
mod tests;
