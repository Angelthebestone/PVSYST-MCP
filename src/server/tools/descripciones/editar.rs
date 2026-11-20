//! Argument descriptions of `editar`.

use super::{entrada, Entrada};

pub(super) const TABLA: &[Entrada] = &[
    entrada(
        "batch_rvt_file",
        "RVT de accion:rvt o plantilla_rvt: ruta o nombre bajo Models/ (alternativa a 'archivo').",
        "RVT of accion:rvt or plantilla_rvt: a path or a name under Models/ (instead of 'archivo').",
    ),
    entrada(
        "nuevo",
        "true: permite que 'campos' añada una clave que el objeto todavía no tiene.",
        "true: lets 'campos' add a key the object does not have yet.",
    ),
    entrada(
        "variables",
        "Variables RVT que 'operacion' añade, quita o reordena; exige 'operacion'.",
        "RVT variables that 'operacion' adds, removes or reorders; requires 'operacion'.",
    ),
];
