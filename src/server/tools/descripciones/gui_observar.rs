//! Argument descriptions of `gui_observar`.

use super::{entrada, Entrada};

pub(super) const TABLA: &[Entrada] = &[
    entrada(
        "completo",
        "ventanas, controles y vista: la forma larga (geometría, esquema) en vez del resumen.",
        "ventanas, controles and vista: the long form (geometry, schema) instead of the summary.",
    ),
    entrada(
        "incluir_uia",
        "controles: añade la lectura UIA; inventario: false la omite; vista la rechaza.",
        "controles: adds the UIA reading; inventario: false skips it; vista refuses it.",
    ),
    entrada(
        "max_nodos",
        "uia: tope de nodos del árbol (150 por defecto); inventario: 400.",
        "uia: cap on tree nodes (150 by default); inventario: 400.",
    ),
    entrada(
        "ocr",
        "leer, capturar y vista: añade el texto dibujado; formulario: false lo omite.",
        "leer, capturar and vista: adds the drawn text; formulario: false skips it.",
    ),
    entrada(
        "profundidad",
        "uia: niveles del árbol (4 por defecto); inventario: 6.",
        "uia: levels of the tree (4 by default); inventario: 6.",
    ),
    entrada(
        "todos",
        "ventanas, controles, formulario e inventario: incluye lo oculto; en ventanas, otros procesos.",
        "ventanas, controles, formulario and inventario: include hidden items; ventanas: other processes.",
    ),
    entrada(
        "visibles",
        "ventanas y controles: false incluye lo oculto; por defecto, solo lo visible.",
        "ventanas and controles: false includes hidden items; by default, only visible ones.",
    ),
];
