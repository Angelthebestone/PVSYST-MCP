//! Agent view of PVsyst forms: each form is a named entity whose fields have
//! stable keys, units and ranges, so an agent reads and fills a form with
//! `{"potencia_nominal_w": 200}` and no captions, OCR or coordinates. The
//! domain (`dominio`, `esquemas`, `plan`) is pure; `campos_vivos`, `vista` and
//! `relleno` talk to the live PVsyst through the GUI adapter.

mod campos_vivos;
mod dominio;
mod esquemas;
mod plan;
mod relleno;
mod ventana;
mod vista;

pub(crate) use relleno::rellenar;
pub(crate) use vista::{listar_esquemas, vista};

#[cfg(test)]
#[path = "../../tests/unit/vista_agente/mod.rs"]
mod tests;
