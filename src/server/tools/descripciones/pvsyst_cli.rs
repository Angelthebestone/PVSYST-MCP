//! Argument descriptions of `pvsyst_cli`.

use super::{entrada, Entrada};

pub(super) const TABLA: &[Entrada] = &[entrada(
    "log_level",
    "Nivel de log de PVsystCLI (-ll), de 0 a 3; un -ll en 'opciones' manda.",
    "PVsystCLI log level (-ll), 0 to 3; an -ll already in 'opciones' wins.",
)];
