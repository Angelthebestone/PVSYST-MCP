//! Text extraction from PVsyst PDF reports (`-rpf`).
//!
//! **Nota de alcance:** Este módulo es un extractor específico para el subconjunto
//! verificado de informes generados por PVsyst (`-rpf`), no un parser general de PDF.
//! Extrae tokens de diagramas de pérdidas, tablas mensuales, resúmenes anuales,
//! evaluación económica y balance de carbono desde streams zlib creados por el motor
//! de reportes de PVsyst.

mod carbono;
mod economia;
mod extract;
mod p50;
mod parse;

pub(crate) use carbono::carbono;
pub(crate) use economia::economia;
pub(crate) use p50::p50;

pub(crate) use extract::tokens;
pub(crate) use parse::{mensual, perdidas, resumen};

#[cfg(test)]
#[path = "../../../tests/unit/simulation/report_pdf/mod.rs"]
mod tests;
