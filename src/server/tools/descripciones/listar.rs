//! Argument descriptions of `listar`.

use super::{entrada, Entrada};

pub(super) const TABLA: &[Entrada] = &[
    entrada(
        "campos",
        "Campos que se conservan en cada elemento; uno desconocido es un error que lista los válidos.",
        "Fields kept in each item; an unknown one is an error that lists the valid ones.",
    ),
    entrada(
        "extension",
        "Filtra que:archivos por extensión; el punto es opcional y no distingue mayúsculas.",
        "Filters que:archivos by extension; the dot is optional and case is ignored.",
    ),
    entrada(
        "include_variants",
        "Por defecto true: en que:proyectos incluye las variantes de cada proyecto.",
        "Defaults to true: que:proyectos lists each project's variants.",
    ),
    entrada(
        "pnom_max",
        "Potencia nominal máxima del módulo en W para que:modulos.",
        "Highest module nominal power in W for que:modulos.",
    ),
    entrada(
        "pnom_min",
        "Potencia nominal mínima del módulo en W para que:modulos.",
        "Lowest module nominal power in W for que:modulos.",
    ),
    entrada(
        "todos",
        "true: con que:objetos no recorta la lista, igual que detalle:completo.",
        "true: with que:objetos, returns every object, like detalle:completo.",
    ),
];
