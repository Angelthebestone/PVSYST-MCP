//! Argument descriptions of `componente`.

use super::{entrada, Entrada};

pub(super) const TABLA: &[Entrada] = &[
    entrada(
        "campos",
        "Objeto campo:valor, sinónimo de 'cambios', aplicado al componente (crear y editar).",
        "field:value object, a synonym of 'cambios', applied to the component (crear, editar).",
    ),
    entrada(
        "sobrescribir",
        "true reemplaza el componente si ya existe (crear, importar, exportar).",
        "true replaces the component when it already exists (crear, importar, exportar).",
    ),
];
