//! Codec and workspace management for PVsyst currency exchange rate table (`Currency.dat`).

mod aplicacion;
mod codec;
mod dominio;

pub(crate) use aplicacion::leer_divisas;

#[cfg(test)]
#[path = "../../tests/unit/divisas/tests.rs"]
mod tests;
