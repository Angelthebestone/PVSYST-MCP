//! `vista` of a call that failed: code, reason, probable cause and what to do.

use super::construccion::{siguiente_de, Contexto};
use super::tipos::{ErrorVista, Estado, Origen, Vista};
use crate::error::{Error, KIND_MISSING_ARG};
use crate::idioma::{Idioma, Texto};
use serde_json::Value;

/// Probable cause and remedy per `Error::kind()`.
const CAUSAS: &[(&str, Texto, Texto)] = &[
    (
        KIND_MISSING_ARG,
        Texto {
            es: "Falta un argumento obligatorio.",
            en: "A required argument is missing.",
        },
        Texto {
            es: "Añade las claves que faltan (ver claves_validas) y repite la llamada.",
            en: "Add the missing keys (see claves_validas) and repeat the call.",
        },
    ),
    (
        "not_found",
        Texto {
            es: "El archivo, proyecto o componente no existe o está fuera del workspace.",
            en: "The file, project or component does not exist or is outside the workspace.",
        },
        Texto {
            es: "Lista los candidatos con listar y repite con el nombre exacto.",
            en: "List the candidates with listar and repeat with the exact name.",
        },
    ),
    (
        "io",
        Texto {
            es: "El sistema operativo rechazó la operación sobre un archivo (ruta, permisos o archivo en uso).",
            en: "The operating system refused the operation on a file (path, permissions or file in use).",
        },
        Texto {
            es: "Comprueba la ruta y que PVsyst no tenga el archivo abierto.",
            en: "Check the path and that PVsyst does not have the file open.",
        },
    ),
    (
        "format",
        Texto {
            es: "El archivo no tiene el formato esperado o la edición lo dañaría.",
            en: "The file does not have the expected format or the edit would damage it.",
        },
        Texto {
            es: "Lee el archivo con leer y comprueba su contenido antes de reintentar.",
            en: "Read the file with leer and check its content before retrying.",
        },
    ),
    (
        "cli",
        Texto {
            es: "PVsystCLI no está disponible, la licencia no permite la operación o la ejecución falló.",
            en: "PVsystCLI is not available, the licence does not allow the operation or the run failed.",
        },
        Texto {
            es: "Ejecuta entorno con accion diagnostico y accion licencia.",
            en: "Run entorno with accion diagnostico and accion licencia.",
        },
    ),
    (
        "gui",
        Texto {
            es: "PVsyst no está abierto, hay un diálogo modal o la ventana no es la esperada.",
            en: "PVsyst is not open, a modal dialogue is up or the window is not the expected one.",
        },
        Texto {
            es: "Mira gui_observar que ventanas; cierra el diálogo con gui_actuar accion cerrar o usa accion reiniciar.",
            en: "Look at gui_observar que ventanas; close the dialogue with gui_actuar accion cerrar or use accion reiniciar.",
        },
    ),
    (
        "network",
        Texto {
            es: "Falló la petición de red a un servicio externo.",
            en: "The network request to an external service failed.",
        },
        Texto {
            es: "Reintenta más tarde o aporta los datos a mano.",
            en: "Retry later or provide the data by hand.",
        },
    ),
    (
        "invalid",
        Texto {
            es: "Un valor no es válido o el estado actual no admite la acción.",
            en: "A value is not valid or the current state does not allow the action.",
        },
        Texto {
            es: "Corrige los argumentos según el motivo y las claves_validas.",
            en: "Fix the arguments as the motivo and claves_validas say.",
        },
    ),
];

const CAUSA_DESCONOCIDA: (Texto, Texto) = (
    Texto {
        es: "La llamada falló por una causa no clasificada.",
        en: "The call failed for an unclassified cause.",
    },
    Texto {
        es: "Lee el motivo y ejecuta entorno con accion diagnostico.",
        en: "Read the motivo and run entorno with accion diagnostico.",
    },
);

/// Summary of a call that did not finish; `{motivo}` is the reason.
const NO_COMPLETADO: Texto = Texto {
    es: "No se completó: {motivo}",
    en: "Not completed: {motivo}",
};

/// Kinds of failure caused by the arguments, for which the valid keys help.
const FALLOS_DE_ENTRADA: &[&str] = &[KIND_MISSING_ARG, "invalid"];

fn causa_de(kind: &str, idioma: Idioma) -> (&'static str, &'static str) {
    let (causa, hacer) = CAUSAS
        .iter()
        .find(|(k, _, _)| *k == kind)
        .map_or(CAUSA_DESCONOCIDA, |(_, causa, hacer)| (*causa, *hacer));
    (causa.en_idioma(idioma), hacer.en_idioma(idioma))
}

/// The `vista` block for `error`; `claves_validas` are the keys the tool accepts.
pub(crate) fn desde_error(ctx: &Contexto<'_>, error: &Error, claves_validas: Vec<String>) -> Vista {
    let kind = error.kind();
    let (causa_probable, que_hacer) = causa_de(kind, ctx.idioma);
    let motivo = error.to_string();
    Vista {
        estado: Estado::Error,
        resumen: NO_COMPLETADO.texto_con(ctx.idioma, &[("motivo", motivo.as_str())]),
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
