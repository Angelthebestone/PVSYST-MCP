//! Legacy (< 6.40) hybrid PAN module decoding and in-place field editing.

pub(crate) mod dominio;
pub(crate) mod edicion;

pub(crate) use dominio::{es_pan_legacy, parse_legacy};
pub(crate) use edicion::parchear_campos_legacy;

#[cfg(test)]
#[path = "../../../tests/unit/components/pan/legacy_edicion_tests.rs"]
mod tests;
