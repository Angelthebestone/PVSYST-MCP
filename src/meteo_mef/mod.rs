//! Codec and workspace management for PVsyst `.MEF` meteo format files.

mod aplicacion;
mod codec;
mod dominio;
mod encoder;
mod legacy;

pub(crate) use aplicacion::{generar_mef_pvgis_nedo, leer_mef, validar_mef};

#[cfg(test)]
#[path = "../../tests/unit/meteo_mef/tests.rs"]
mod tests;
