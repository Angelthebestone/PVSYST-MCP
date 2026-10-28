//! `componente`: module and inverter files of the workspace.

use super::{entrada, paso, Entrada, Paso, DE_ARCHIVO, VER_COMPONENTES, VER_MODULOS};

const LEER: Paso = paso("leer", "Leer el componente").tomando(DE_ARCHIVO);

pub(super) const ENTRADAS: &[Entrada] = &[
    entrada(
        "componente",
        "crear",
        "Componente creado a partir de una plantilla.",
    )
    .pista("proyecto crear {modulo|inversor}")
    .luego(&[VER_COMPONENTES, VER_MODULOS]),
    entrada("componente", "editar", "Componente editado.").luego(&[LEER, VER_MODULOS]),
    entrada("componente", "borrar", "Componente borrado.").luego(&[VER_MODULOS]),
    entrada(
        "componente",
        "importar",
        "Componente importado al workspace.",
    )
    .luego(&[VER_MODULOS]),
    entrada(
        "componente",
        "exportar",
        "Componente exportado fuera del workspace.",
    )
    .luego(&[VER_MODULOS]),
    entrada(
        "componente",
        "crear_desde_ficha",
        "Componente creado desde los datos de una ficha técnica.",
    )
    .pista("proyecto crear {modulo|inversor}")
    .luego(&[VER_COMPONENTES, VER_MODULOS]),
];
