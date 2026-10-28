//! `vista` of a call that failed: code, reason, probable cause and what to do.

use super::construccion::{siguiente_de, Contexto};
use super::tipos::{ErrorVista, Estado, Origen, Vista};
use crate::error::{Error, KIND_MISSING_ARG};
use serde_json::Value;

/// Probable cause and remedy per `Error::kind()`.
const CAUSAS: &[(&str, &str, &str)] = &[
    (
        KIND_MISSING_ARG,
        "Falta un argumento obligatorio.",
        "Añade las claves que faltan (ver claves_validas) y repite la llamada.",
    ),
    (
        "not_found",
        "El archivo, proyecto o componente no existe o está fuera del workspace.",
        "Lista los candidatos con listar y repite con el nombre exacto.",
    ),
    (
        "io",
        "El sistema operativo rechazó la operación sobre un archivo (ruta, permisos o archivo en uso).",
        "Comprueba la ruta y que PVsyst no tenga el archivo abierto.",
    ),
    (
        "format",
        "El archivo no tiene el formato esperado o la edición lo dañaría.",
        "Lee el archivo con leer y comprueba su contenido antes de reintentar.",
    ),
    (
        "cli",
        "PVsystCLI no está disponible, la licencia no permite la operación o la ejecución falló.",
        "Ejecuta entorno con accion diagnostico y accion licencia.",
    ),
    (
        "gui",
        "PVsyst no está abierto, hay un diálogo modal o la ventana no es la esperada.",
        "Mira gui_observar que ventanas; cierra el diálogo con gui_actuar accion cerrar o usa accion reiniciar.",
    ),
    (
        "network",
        "Falló la petición de red a un servicio externo.",
        "Reintenta más tarde o aporta los datos a mano.",
    ),
    (
        "invalid",
        "Un valor no es válido o el estado actual no admite la acción.",
        "Corrige los argumentos según el motivo y las claves_validas.",
    ),
];

const CAUSA_DESCONOCIDA: (&str, &str) = (
    "La llamada falló por una causa no clasificada.",
    "Lee el motivo y ejecuta entorno con accion diagnostico.",
);

/// Kinds of failure caused by the arguments, for which the valid keys help.
const FALLOS_DE_ENTRADA: &[&str] = &[KIND_MISSING_ARG, "invalid"];

fn causa_de(kind: &str) -> (&'static str, &'static str) {
    CAUSAS
        .iter()
        .find(|(k, _, _)| *k == kind)
        .map_or(CAUSA_DESCONOCIDA, |(_, causa, hacer)| (*causa, *hacer))
}

/// The `vista` block for `error`; `claves_validas` are the keys the tool accepts.
pub(crate) fn desde_error(ctx: &Contexto<'_>, error: &Error, claves_validas: Vec<String>) -> Vista {
    let kind = error.kind();
    let (causa_probable, que_hacer) = causa_de(kind);
    let motivo = error.to_string();
    Vista {
        estado: Estado::Error,
        resumen: format!("No se completó: {motivo}"),
        origen: Origen::Mcp,
        campos: Vec::new(),
        avisos: Vec::new(),
        siguiente: siguiente_de(ctx, Estado::Error, &Value::Null),
        recortado: Vec::new(),
        error: Some(ErrorVista {
            codigo: kind.to_string(),
            motivo,
            causa_probable,
            que_hacer,
            claves_validas: if FALLOS_DE_ENTRADA.contains(&kind) {
                claves_validas
            } else {
                Vec::new()
            },
        }),
    }
}
