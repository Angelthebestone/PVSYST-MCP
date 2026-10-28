//! `sitio`: sites, weather series and horizons.

use super::{entrada, paso, Entrada, Paso, DE_ARCHIVO, VER_ARCHIVOS};

const LEER: Paso = paso("leer", "Leer el archivo generado").tomando(DE_ARCHIVO);

pub(super) const ENTRADAS: &[Entrada] = &[
    entrada(
        "sitio",
        "crear",
        "Sitio creado con la meteorología de la ubicación.",
    )
    .pista("proyecto crear {plantilla,nombre_nuevo}")
    .luego(&[LEER, VER_ARCHIVOS]),
    entrada(
        "sitio",
        "convertir_meteo",
        "Archivo meteorológico convertido al formato de PVsyst.",
    )
    .luego(&[LEER, VER_ARCHIVOS]),
    entrada(
        "sitio",
        "geometria_solar",
        "Geometría solar calculada para la ubicación.",
    )
    .luego(&[VER_ARCHIVOS]),
    entrada("sitio", "horizonte_pvgis", "Horizonte descargado de PVGIS.").luego(&[VER_ARCHIVOS]),
    entrada("sitio", "meteo_pvgis", "Meteorología descargada de PVGIS.")
        .luego(&[LEER, VER_ARCHIVOS]),
    entrada(
        "sitio",
        "escribir_horizonte",
        "Horizonte escrito en el sitio.",
    )
    .luego(&[LEER, VER_ARCHIVOS]),
    entrada(
        "sitio",
        "escribir_serie",
        "Serie meteorológica escrita en el sitio.",
    )
    .luego(&[LEER, VER_ARCHIVOS]),
];
