//! High-level "easy mode": environment preflight, a one-call end-to-end design
//! workflow, and headline KPI extraction.
//!
//! The point is that an LLM (or a user who does not know PVsyst) can accomplish
//! a full study with **one** tool call instead of orchestrating a dozen, which
//! also removes most of the per-turn latency.

mod baterias;
mod diagnostico;
mod dimensionado;
mod diseno;
mod fallo_simulacion;
mod foto_proyecto;
mod lectura_argumentos;
mod meteonorm;
mod modulo_de_plantilla;
mod parametros;
mod pasos;
pub(crate) mod proyecto;
mod resultado;
mod serie_plantilla;
mod simulacion;
mod sitio;
mod sitio_del_proyecto;

pub(crate) use diseno::disenar_sistema;

#[cfg(test)]
#[path = "../../tests/unit/easy_mode/tests.rs"]
mod tests;
#[cfg(test)]
#[path = "../../tests/unit/easy_mode/pasos.rs"]
mod tests_pasos;

pub(crate) use diagnostico::preflight;
pub(crate) use simulacion::leer_kpis;
