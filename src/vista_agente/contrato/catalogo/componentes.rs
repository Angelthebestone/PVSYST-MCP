//! `componente`: module and inverter files of the workspace.

use super::{entrada, paso, Entrada, Paso, DE_ARCHIVO, VER_COMPONENTES, VER_MODULOS};
use crate::idioma::Texto;

const LEER: Paso = paso(
    "leer",
    Texto {
        es: "Leer el componente",
        en: "Read the component",
    },
)
.tomando(DE_ARCHIVO);

pub(super) const ENTRADAS: &[Entrada] = &[
    entrada(
        "componente",
        "crear",
        Texto {
            es: "Componente creado a partir de una plantilla.",
            en: "Component created from a template.",
        },
    )
    .pista(Texto {
        es: "proyecto crear {modulo|inversor}",
        en: "proyecto crear {modulo|inversor}",
    })
    .luego(&[VER_COMPONENTES, VER_MODULOS]),
    entrada(
        "componente",
        "editar",
        Texto {
            es: "Componente editado.",
            en: "Component edited.",
        },
    )
    .luego(&[LEER, VER_MODULOS]),
    entrada(
        "componente",
        "borrar",
        Texto {
            es: "Componente borrado.",
            en: "Component deleted.",
        },
    )
    .luego(&[VER_MODULOS]),
    entrada(
        "componente",
        "importar",
        Texto {
            es: "Componente importado al workspace.",
            en: "Component imported into the workspace.",
        },
    )
    .luego(&[VER_MODULOS]),
    entrada(
        "componente",
        "exportar",
        Texto {
            es: "Componente exportado fuera del workspace.",
            en: "Component exported outside the workspace.",
        },
    )
    .luego(&[VER_MODULOS]),
    entrada(
        "componente",
        "crear_desde_ficha",
        Texto {
            es: "Componente creado desde los datos de una ficha técnica.",
            en: "Component created from the data of a datasheet.",
        },
    )
    .pista(Texto {
        es: "proyecto crear {modulo|inversor}",
        en: "proyecto crear {modulo|inversor}",
    })
    .luego(&[VER_COMPONENTES, VER_MODULOS]),
];
