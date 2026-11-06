//! Client-facing wording of the MCP schema annotations, kept in one place so
//! every builder emits the same text and a wording change stays a one-line edit.

use crate::idioma::Texto;

/// Suffix appended to a property description when the argument is required.
pub(super) const SUFIJO_REQUERIDO: Texto = Texto {
    es: " Requerido.",
    en: " Required.",
};

/// Prefix introducing the canonical values of a discriminator property.
pub(super) const PREFIJO_VALORES: Texto = Texto {
    es: "Valores: ",
    en: "Values: ",
};

/// Prefix introducing the required keys of each admitted action.
pub(super) const PREFIJO_REQUERIDOS_POR_ACCION: Texto = Texto {
    es: " Requeridos por acción: ",
    en: " Required by action: ",
};

/// Description of the trusted-client `workspace` confinement override.
pub(super) const DESCRIPCION_WORKSPACE: Texto = Texto {
    es: "Ruta del workspace que reemplaza la raíz de confinamiento para esta llamada. Requiere cliente MCP de confianza.",
    en: "Workspace path that replaces the confinement root for this call. Requires a trusted MCP client.",
};
