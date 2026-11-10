//! `sitio`: sites, weather series and horizons.

use super::{entrada, paso, Entrada, Paso, DE_ARCHIVO, VER_ARCHIVOS};
use crate::idioma::Texto;

const LEER: Paso = paso(
    "leer",
    Texto {
        es: "Leer el archivo generado",
        en: "Read the generated file",
    },
)
.tomando(DE_ARCHIVO);

pub(super) const ENTRADAS: &[Entrada] = &[
    entrada(
        "sitio",
        "crear",
        Texto {
            es: "Sitio creado con la meteorología de la ubicación.",
            en: "Site created with the weather data of the location.",
        },
    )
    .pista(Texto {
        es: "proyecto crear {plantilla,nombre_nuevo}",
        en: "proyecto crear {plantilla,nombre_nuevo}",
    })
    .luego(&[LEER, VER_ARCHIVOS]),
    entrada(
        "sitio",
        "convertir_meteo",
        Texto {
            es: "Archivo meteorológico convertido al formato de PVsyst.",
            en: "Weather file converted to the PVsyst format.",
        },
    )
    .luego(&[LEER, VER_ARCHIVOS]),
    entrada(
        "sitio",
        "geometria_solar",
        Texto {
            es: "Geometría solar calculada para la ubicación.",
            en: "Solar geometry computed for the location.",
        },
    )
    .luego(&[VER_ARCHIVOS]),
    entrada(
        "sitio",
        "horizonte_pvgis",
        Texto {
            es: "Horizonte descargado de PVGIS.",
            en: "Horizon downloaded from PVGIS.",
        },
    )
    .luego(&[VER_ARCHIVOS]),
    entrada(
        "sitio",
        "meteo_pvgis",
        Texto {
            es: "Meteorología descargada de PVGIS.",
            en: "Weather data downloaded from PVGIS.",
        },
    )
    .luego(&[LEER, VER_ARCHIVOS]),
    entrada(
        "sitio",
        "escribir_horizonte",
        Texto {
            es: "Horizonte escrito en el sitio.",
            en: "Horizon written into the site.",
        },
    )
    .luego(&[LEER, VER_ARCHIVOS]),
    entrada(
        "sitio",
        "escribir_serie",
        Texto {
            es: "Serie meteorológica escrita en el sitio.",
            en: "Weather series written into the site.",
        },
    )
    .luego(&[LEER, VER_ARCHIVOS]),
];
