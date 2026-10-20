//! Simulation result cache. The key is a FNV-1a 64-bit hash over every
//! project and variant files, the input files passed as options, and the sites,
//! meteo files and components they reference. A hit copies the stored CSV/PDF and
//! skips the trial execution.

mod key;
mod store;

pub(crate) use key::{clave, ClaveCache};
pub(crate) use store::{buscar, guardar};

#[cfg(test)]
#[path = "../../../tests/unit/simulation/cache/mod.rs"]
mod tests;
