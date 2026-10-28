//! `proyecto`: lifecycle and configuration of projects and variants.

use super::{
    entrada, paso, Entrada, Paso, P50_P90, REVISAR, SIMULAR, VER_HISTORIAL, VER_PROYECTOS,
};

/// The renamed project: the new name, not the old one the request also carries.
const REVISAR_RENOMBRADO: Paso = paso("proyecto", "Revisar el proyecto renombrado")
    .con(&[("accion", "doctor")])
    .tomando(&[("proyecto", &["nombre_nuevo"])]);

pub(super) const ENTRADAS: &[Entrada] = &[
    entrada("proyecto", "crear", "Proyecto creado desde una plantilla.")
        .pista("proyecto dimensionar {proyecto,n_mod_serie,n_strings}")
        .luego(&[REVISAR, VER_PROYECTOS]),
    entrada("proyecto", "eliminar", "Proyecto eliminado.").luego(&[VER_PROYECTOS]),
    entrada("proyecto", "renombrar", "Proyecto renombrado.")
        .luego(&[REVISAR_RENOMBRADO, VER_PROYECTOS]),
    entrada(
        "proyecto",
        "crear_variante",
        "Variante creada a partir de otra.",
    )
    .luego(&[REVISAR, VER_PROYECTOS]),
    entrada(
        "proyecto",
        "dimensionar",
        "Proyecto dimensionado con módulos en serie y strings.",
    )
    .pista("proyecto doctor {proyecto}")
    .luego(&[REVISAR, VER_PROYECTOS]),
    entrada(
        "proyecto",
        "doctor",
        "Proyecto revisado: coherencia de la variante y sus archivos.",
    )
    .pista("simular {proyecto}")
    .luego(&[SIMULAR, VER_PROYECTOS]),
    entrada("proyecto", "anadir", "Elemento añadido a la variante.")
        .luego(&[REVISAR, VER_HISTORIAL]),
    entrada("proyecto", "seguidor", "Variante convertida a seguidor.")
        .luego(&[REVISAR, VER_HISTORIAL]),
    entrada(
        "proyecto",
        "limite_inyeccion",
        "Límite de inyección a red aplicado.",
    )
    .luego(&[REVISAR, VER_HISTORIAL]),
    entrada(
        "proyecto",
        "almacenamiento",
        "Almacenamiento con baterías configurado.",
    )
    .luego(&[REVISAR, VER_HISTORIAL]),
    entrada("proyecto", "perdidas", "Pérdidas detalladas configuradas.")
        .luego(&[REVISAR, VER_HISTORIAL]),
    entrada("proyecto", "bombeo", "Sistema de bombeo configurado.")
        .luego(&[REVISAR, VER_HISTORIAL]),
    entrada("proyecto", "economia", "Evaluación económica configurada.")
        .luego(&[REVISAR, VER_HISTORIAL]),
    entrada(
        "proyecto",
        "configurar",
        "Cambios de variante aplicados en una sola llamada.",
    )
    .luego(&[REVISAR, VER_HISTORIAL]),
    entrada(
        "proyecto",
        "p50_p90",
        "Parámetros de incertidumbre P50/P90 configurados.",
    )
    .luego(&[P50_P90, VER_HISTORIAL]),
    entrada(
        "proyecto",
        "limpiar_temporales",
        "Archivos temporales del MCP eliminados.",
    )
    .luego(&[VER_PROYECTOS]),
];
