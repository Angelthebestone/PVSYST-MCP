//! `workspace`, `listar`, `leer`, `editar` and `escribir_archivo`.

use super::{
    entrada, paso, Entrada, Paso, DE_ARCHIVO, REVISAR, VER_ARCHIVOS, VER_HISTORIAL, VER_MODULOS,
    VER_PROYECTOS,
};
use crate::idioma::Texto;

const LEER: Paso = paso(
    "leer",
    Texto {
        es: "Leer el archivo",
        en: "Read the file",
    },
)
.tomando(DE_ARCHIVO);
const VER_OBJETOS: Paso = paso(
    "listar",
    Texto {
        es: "Ver los objetos del archivo",
        en: "See the objects of the file",
    },
)
.con(&[("que", "objetos")])
.tomando(DE_ARCHIVO);

pub(super) const ENTRADAS: &[Entrada] = &[
    entrada(
        "workspace",
        "crear",
        Texto {
            es: "Workspace creado.",
            en: "Workspace created.",
        },
    )
    .luego(&[VER_PROYECTOS, VER_HISTORIAL]),
    entrada(
        "workspace",
        "exportar",
        Texto {
            es: "Workspace exportado a un archivo.",
            en: "Workspace exported to a file.",
        },
    )
    .luego(&[VER_PROYECTOS, VER_HISTORIAL]),
    entrada(
        "workspace",
        "importar",
        Texto {
            es: "Workspace importado.",
            en: "Workspace imported.",
        },
    )
    .luego(&[VER_PROYECTOS, VER_HISTORIAL]),
    entrada(
        "workspace",
        "exportar_proyecto",
        Texto {
            es: "Proyecto exportado a un archivo.",
            en: "Project exported to a file.",
        },
    )
    .luego(&[VER_PROYECTOS, VER_HISTORIAL]),
    entrada(
        "workspace",
        "historial",
        Texto {
            es: "Historial de cambios del workspace.",
            en: "Workspace change history.",
        },
    )
    .luego(&[VER_PROYECTOS]),
    entrada(
        "workspace",
        "restaurar",
        Texto {
            es: "Cambio restaurado desde el historial.",
            en: "Change restored from the history.",
        },
    )
    .luego(&[VER_HISTORIAL, VER_PROYECTOS]),
    entrada(
        "listar",
        "proyectos",
        Texto {
            es: "Proyectos del workspace listados.",
            en: "Workspace projects listed.",
        },
    )
    .luego(&[VER_ARCHIVOS, VER_MODULOS]),
    entrada(
        "listar",
        "componentes",
        Texto {
            es: "Componentes del tipo pedido listados.",
            en: "Components of the requested kind listed.",
        },
    )
    .luego(&[VER_MODULOS, VER_PROYECTOS]),
    entrada(
        "listar",
        "modulos",
        Texto {
            es: "Módulos fotovoltaicos listados.",
            en: "Photovoltaic modules listed.",
        },
    )
    .luego(&[VER_ARCHIVOS, VER_PROYECTOS]),
    entrada(
        "listar",
        "archivos",
        Texto {
            es: "Archivos del workspace listados.",
            en: "Workspace files listed.",
        },
    )
    .luego(&[VER_PROYECTOS, VER_MODULOS]),
    entrada(
        "listar",
        "objetos",
        Texto {
            es: "Objetos del archivo PVsyst listados.",
            en: "Objects of the PVsyst file listed.",
        },
    )
    .luego(&[LEER, VER_PROYECTOS]),
    entrada(
        "leer",
        "",
        Texto {
            es: "Archivo de PVsyst leído.",
            en: "PVsyst file read.",
        },
    )
    .luego(&[VER_OBJETOS, VER_PROYECTOS]),
    entrada(
        "editar",
        "campos",
        Texto {
            es: "Campos editados con copia en el historial.",
            en: "Fields edited with a copy in the history.",
        },
    )
    .pista(Texto {
        es: "proyecto doctor",
        en: "proyecto doctor",
    })
    .luego(&[REVISAR, LEER, VER_HISTORIAL]),
    entrada(
        "editar",
        "insertar_objeto",
        Texto {
            es: "Objeto insertado en el archivo.",
            en: "Object inserted into the file.",
        },
    )
    .luego(&[LEER, VER_HISTORIAL]),
    entrada(
        "editar",
        "eliminar_objeto",
        Texto {
            es: "Objeto eliminado del archivo.",
            en: "Object deleted from the file.",
        },
    )
    .luego(&[LEER, VER_HISTORIAL]),
    entrada(
        "editar",
        "rvt",
        Texto {
            es: "Operación sobre el archivo RVT aplicada.",
            en: "Operation on the RVT file applied.",
        },
    )
    .luego(&[VER_HISTORIAL]),
    entrada(
        "editar",
        "plantilla_rvt",
        Texto {
            es: "Plantilla RVT generada.",
            en: "RVT template generated.",
        },
    )
    .luego(&[VER_HISTORIAL]),
    entrada(
        "escribir_archivo",
        "",
        Texto {
            es: "Archivo escrito en el workspace con copia en el historial.",
            en: "File written into the workspace with a copy in the history.",
        },
    )
    .luego(&[LEER, VER_HISTORIAL]),
];
