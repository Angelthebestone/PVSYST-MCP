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

pub(crate) use carbono::{carbono, carbono_de};
pub(crate) use caso::{caso, con_cabecera, Caso};
pub(crate) use comparacion::comparar;
pub(crate) use economia::{economia, economia_de};
pub(crate) use entregable::{entregable, mensual, mensual_de, perdidas, perdidas_de};
pub(crate) use incertidumbre::{p50_p90, p50_p90_de};

#[cfg(test)]
#[path = "../../../tests/unit/simulation/engineering/mod.rs"]
mod tests;
