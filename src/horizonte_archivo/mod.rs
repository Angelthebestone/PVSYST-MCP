//! Codec and workspace management for PVsyst `.HOR` horizon files.
//!
//! `.HOR` files store far shading horizon lines as a two-line ASCII header
//! followed by binary records of Delphi `Real48` floating-point numbers.

mod aplicacion;
mod codec;
mod dominio;

pub(crate) use aplicacion::{escribir_horizonte, leer_horizonte};
#[cfg(test)]
pub(crate) use codec::{encode_horizonte, parse_horizonte};
#[cfg(test)]
pub(crate) use dominio::{Horizonte, PuntoHorizonte};

#[cfg(test)]
#[path = "../../tests/unit/horizonte_archivo/tests.rs"]
mod tests;
