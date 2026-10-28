//! `proyecto`: lifecycle and configuration of projects and variants.

use super::{entrada, paso, Entrada, Paso, DE_PROYECTO};

const VER_PROYECTOS: Paso =
    paso("listar", "Ver los proyectos del workspace").con(&[("que", "proyectos")]);
const REVISAR: Paso = paso("proyecto", "Revisar que el proyecto es coherente")
    .con(&[("accion", "doctor")])
    .tomando(DE_PROYECTO);
const SIMULAR: Paso = paso("simular", "Simular el proyecto")
    .con(&[("accion", "ejecutar")])
    .tomando(DE_PROYECTO);
const P50_P90: Paso = paso("resultados", "Ver la producción P50/P90")
    .con(&[("que", "p50_p90")])
    .tomando(DE_PROYECTO);
const DESHACER: Paso =
    paso("workspace", "Ver el historial para poder revertir").con(&[("accion", "historial")]);

pub(super) const ENTRADAS: &[Entrada] = &[
    entrada("proyecto", "crear", "Proyecto creado desde una plantilla.")
        .pista("proyecto dimensionar {proyecto,n_mod_serie,n_strings}")
        .luego(&[REVISAR, VER_PROYECTOS]),
    entrada("proyecto", "eliminar", "Proyecto eliminado.").luego(&[VER_PROYECTOS]),
    entrada("proyecto", "renombrar", "Proyecto renombrado.").luego(&[REVISAR, VER_PROYECTOS]),
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
    entrada("proyecto", "anadir", "Elemento añadido a la variante.").luego(&[REVISAR, DESHACER]),
    entrada("proyecto", "seguidor", "Variante convertida a seguidor.").luego(&[REVISAR, DESHACER]),
    entrada(
        "proyecto",
        "limite_inyeccion",
        "Límite de inyección a red aplicado.",
    )
    .luego(&[REVISAR, DESHACER]),
    entrada(
        "proyecto",
        "almacenamiento",
        "Almacenamiento con baterías configurado.",
    )
    .luego(&[REVISAR, DESHACER]),
    entrada("proyecto", "perdidas", "Pérdidas detalladas configuradas.")
        .luego(&[REVISAR, DESHACER]),
    entrada("proyecto", "bombeo", "Sistema de bombeo configurado.").luego(&[REVISAR, DESHACER]),
    entrada("proyecto", "economia", "Evaluación económica configurada.")
        .luego(&[REVISAR, DESHACER]),
    entrada(
        "proyecto",
        "configurar",
        "Cambios de variante aplicados en una sola llamada.",
    )
    .luego(&[REVISAR, DESHACER]),
    entrada(
        "proyecto",
        "p50_p90",
        "Parámetros de incertidumbre P50/P90 configurados.",
    )
    .luego(&[P50_P90, DESHACER]),
    entrada(
        "proyecto",
        "limpiar_temporales",
        "Archivos temporales del MCP eliminados.",
    )
    .luego(&[VER_PROYECTOS]),
];
