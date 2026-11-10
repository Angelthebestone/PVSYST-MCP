//! `proyecto`: lifecycle and configuration of projects and variants.

use super::{
    entrada, paso, Entrada, Paso, P50_P90, REVISAR, SIMULAR, VER_HISTORIAL, VER_PROYECTOS,
};
use crate::idioma::Texto;

/// The renamed project: the new name, not the old one the request also carries.
const REVISAR_RENOMBRADO: Paso = paso(
    "proyecto",
    Texto {
        es: "Revisar el proyecto renombrado",
        en: "Check the renamed project",
    },
)
.con(&[("accion", "doctor")])
.tomando(&[("proyecto", &["nombre_nuevo"])]);

pub(super) const ENTRADAS: &[Entrada] = &[
    entrada(
        "proyecto",
        "crear",
        Texto {
            es: "Proyecto creado desde una plantilla.",
            en: "Project created from a template.",
        },
    )
    .pista(Texto {
        es: "proyecto dimensionar {proyecto,n_mod_serie,n_strings}",
        en: "proyecto dimensionar {proyecto,n_mod_serie,n_strings}",
    })
    .luego(&[REVISAR, VER_PROYECTOS]),
    entrada(
        "proyecto",
        "eliminar",
        Texto {
            es: "Proyecto eliminado.",
            en: "Project deleted.",
        },
    )
    .luego(&[VER_PROYECTOS]),
    entrada(
        "proyecto",
        "renombrar",
        Texto {
            es: "Proyecto renombrado.",
            en: "Project renamed.",
        },
    )
    .luego(&[REVISAR_RENOMBRADO, VER_PROYECTOS]),
    entrada(
        "proyecto",
        "crear_variante",
        Texto {
            es: "Variante creada a partir de otra.",
            en: "Variant created from another one.",
        },
    )
    .luego(&[REVISAR, VER_PROYECTOS]),
    entrada(
        "proyecto",
        "dimensionar",
        Texto {
            es: "Proyecto dimensionado con módulos en serie y strings.",
            en: "Project sized with modules in series and strings.",
        },
    )
    .pista(Texto {
        es: "proyecto doctor {proyecto}",
        en: "proyecto doctor {proyecto}",
    })
    .luego(&[REVISAR, VER_PROYECTOS]),
    entrada(
        "proyecto",
        "doctor",
        Texto {
            es: "Proyecto revisado: coherencia de la variante y sus archivos.",
            en: "Project checked: coherence of the variant and its files.",
        },
    )
    .pista(Texto {
        es: "simular {proyecto}",
        en: "simular {proyecto}",
    })
    .luego(&[SIMULAR, VER_PROYECTOS]),
    entrada(
        "proyecto",
        "anadir",
        Texto {
            es: "Elemento añadido a la variante.",
            en: "Element added to the variant.",
        },
    )
    .luego(&[REVISAR, VER_HISTORIAL]),
    entrada(
        "proyecto",
        "seguidor",
        Texto {
            es: "Variante convertida a seguidor.",
            en: "Variant converted to a tracker.",
        },
    )
    .luego(&[REVISAR, VER_HISTORIAL]),
    entrada(
        "proyecto",
        "limite_inyeccion",
        Texto {
            es: "Límite de inyección a red aplicado.",
            en: "Grid injection limit applied.",
        },
    )
    .luego(&[REVISAR, VER_HISTORIAL]),
    entrada(
        "proyecto",
        "almacenamiento",
        Texto {
            es: "Almacenamiento con baterías configurado.",
            en: "Battery storage configured.",
        },
    )
    .luego(&[REVISAR, VER_HISTORIAL]),
    entrada(
        "proyecto",
        "perdidas",
        Texto {
            es: "Pérdidas detalladas configuradas.",
            en: "Detailed losses configured.",
        },
    )
    .luego(&[REVISAR, VER_HISTORIAL]),
    entrada(
        "proyecto",
        "bombeo",
        Texto {
            es: "Sistema de bombeo configurado.",
            en: "Pumping system configured.",
        },
    )
    .luego(&[REVISAR, VER_HISTORIAL]),
    entrada(
        "proyecto",
        "economia",
        Texto {
            es: "Evaluación económica configurada.",
            en: "Economic evaluation configured.",
        },
    )
    .luego(&[REVISAR, VER_HISTORIAL]),
    entrada(
        "proyecto",
        "configurar",
        Texto {
            es: "Cambios de variante aplicados en una sola llamada.",
            en: "Variant changes applied in a single call.",
        },
    )
    .luego(&[REVISAR, VER_HISTORIAL]),
    entrada(
        "proyecto",
        "p50_p90",
        Texto {
            es: "Parámetros de incertidumbre P50/P90 configurados.",
            en: "P50/P90 uncertainty parameters configured.",
        },
    )
    .luego(&[P50_P90, VER_HISTORIAL]),
    entrada(
        "proyecto",
        "limpiar_temporales",
        Texto {
            es: "Archivos temporales del MCP eliminados.",
            en: "MCP temporary files deleted.",
        },
    )
    .luego(&[VER_PROYECTOS]),
];
