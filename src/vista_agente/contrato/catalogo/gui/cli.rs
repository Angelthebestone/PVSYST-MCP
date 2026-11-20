//! `pvsyst_cli`: the passthrough to PVsystCLI, whose next step is the diagnostic.

use super::super::{entrada, Entrada};
use super::pasos::VER_ENTORNO;
use crate::idioma::Texto;

pub(super) const PVSYST_CLI: Entrada = entrada(
    "pvsyst_cli",
    "",
    Texto {
        es: "Comando de PVsystCLI ejecutado.",
        en: "PVsystCLI command run.",
    },
)
.luego(&[VER_ENTORNO]);
