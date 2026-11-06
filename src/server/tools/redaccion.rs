//! Client-facing wording of the MCP schema annotations, kept in one place so
//! every builder emits the same text and a wording change stays a one-line edit.

/// Suffix appended to a property description when the argument is required.
pub(super) const SUFIJO_REQUERIDO: &str = " Required.";

/// Prefix introducing the canonical values of a discriminator property.
pub(super) const PREFIJO_VALORES: &str = "Values: ";

/// Prefix introducing the required keys of each admitted action.
pub(super) const PREFIJO_REQUERIDOS_POR_ACCION: &str = " Required by action: ";

/// Description of the trusted-client `workspace` confinement override.
pub(super) const DESCRIPCION_WORKSPACE: &str = "Workspace path that replaces the confinement root for this call. Requires a trusted MCP client.";
