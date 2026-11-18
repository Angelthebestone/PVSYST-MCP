//! Argument descriptions of `leer`.

use super::{entrada, Entrada};

pub(super) const TABLA: &[Entrada] = &[
    entrada(
        "base64",
        "true: lee el archivo como bytes crudos y los devuelve en base64.",
        "true: reads the file as raw bytes and returns them base64-encoded.",
    ),
    entrada(
        "max_bytes",
        "Bytes máximos devueltos (por defecto 32768; tope 10485760); serie:true lo ignora.",
        "Maximum bytes returned (default 32768; ceiling 10485760); serie:true ignores it.",
    ),
    entrada(
        "offset",
        "Byte donde empieza la ventana; con serie:true, primera hora devuelta (0 por defecto).",
        "Byte where the window starts; with serie:true, first hourly record (0 by default).",
    ),
];
