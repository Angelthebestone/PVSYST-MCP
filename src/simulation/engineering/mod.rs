//! Engineering results derived from PVsyst result files: loss and monthly
//! tables, variant comparison, P50/P90 uncertainty, carbon balance, economics
//! and the Markdown deliverable. Figures come from the result CSV, its PDF
//! report and the project file; no simulation is run.

mod carbono;
mod caso;
mod comparacion;
mod economia;
mod entregable;
mod incertidumbre;
mod supuestos;

pub(crate) use carbono::carbono;
pub(crate) use caso::{caso, con_cabecera};
pub(crate) use comparacion::comparar;
pub(crate) use economia::economia;
pub(crate) use entregable::{entregable, mensual, perdidas};
pub(crate) use incertidumbre::p50_p90;

#[cfg(test)]
#[path = "../../../tests/unit/simulation/engineering/mod.rs"]
mod tests;
