//! Argument descriptions of `entorno`.

use super::{entrada, Entrada};

pub(super) const TABLA: &[Entrada] = &[
    entrada(
        "n",
        "Cuántos logs CLI recientes resume logs_cli (por defecto 1; tope 20).",
        "How many recent CLI logs logs_cli summarises (default 1; at most 20).",
    ),
    entrada(
        "solo_errores",
        "true: el resumen de logs_cli conserva solo las líneas de error.",
        "true: the logs_cli summary keeps only the error lines.",
    ),
];
