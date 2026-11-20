//! Argument descriptions of `escribir_archivo`.

use super::{entrada, Entrada};

pub(super) const TABLA: &[Entrada] = &[
    entrada(
        "base64",
        "true: 'contenido' es base64 y se decodifica antes de escribir.",
        "true: 'contenido' is base64 and is decoded before writing.",
    ),
    entrada(
        "sobrescribir",
        "true reemplaza un archivo existente; el historial guarda la versión anterior.",
        "true replaces an existing file; the history keeps the previous version.",
    ),
];
