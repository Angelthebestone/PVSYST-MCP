//! Text extraction from PVsyst PDF reports (`-rpf`).
//!
//! **Nota de alcance:** Este módulo es un extractor específico para el subconjunto
//! verificado de informes generados por PVsyst (`-rpf`), no un parser general de PDF.
//! Extrae tokens de diagramas de pérdidas, tablas mensuales y resúmenes anuales
//! desde streams zlib creados por el motor de reportes de PVsyst.

mod extract;
mod parse;

pub(crate) use extract::tokens;
pub(crate) use parse::{mensual, perdidas, resumen};

#[cfg(test)]
#[path = "../../../tests/unit/simulation/report_pdf/mod.rs"]
mod tests;
