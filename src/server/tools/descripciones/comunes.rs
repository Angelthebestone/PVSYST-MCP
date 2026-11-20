//! Keys that every tool that declares them reads the same way.

use super::{entrada, Entrada};

pub(super) const TABLA: &[Entrada] = &[
    entrada(
        "detalle",
        "Elementos por lista: resumen 13 (por defecto), normal 50, completo todos.",
        "Items per list: resumen 13 (default), normal 50, completo all.",
    ),
    entrada(
        "permitir_fuera_del_workspace",
        "true permite rutas fuera del workspace (por defecto, confinadas).",
        "true allows paths outside the workspace (default: confined).",
    ),
    entrada(
        "vista",
        "false omite vista solo al terminar la llamada; errores y dry_run la conservan (true por defecto).",
        "false drops vista only on a finished call; errors and dry_run keep it (default true).",
    ),
];
