//! Argument descriptions of `workspace`.

use super::{entrada, Entrada};

pub(super) const TABLA: &[Entrada] = &[
    entrada(
        "id",
        "Entrada del historial que restaura accion:restaurar (el id que dio 'historial').",
        "History entry restored by accion:restaurar (the id 'historial' returned).",
    ),
    entrada(
        "n",
        "Cuántas entradas del historial devolver, de la más reciente atrás (por defecto 20).",
        "How many history entries to return, newest first (default 20).",
    ),
];
