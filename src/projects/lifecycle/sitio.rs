//! The site object of a project: the `pvSite` of a `.SIT` file becomes the project's
//! `SitePrj`. Shared by `proyecto crear` and `disenar_sistema`.

use crate::error::Result;
use crate::format::insert;
use std::path::Path;

/// `prj` with its `SitePrj` object replaced by the `pvSite` of the `.SIT` at `sit_file`.
pub(crate) fn insertar_sitio(prj: &[u8], sit_file: &Path) -> Result<Vec<u8>> {
    let sit = crate::workspace::leer_bytes(sit_file)?;
    let bloque = insert::extraer_objeto(&sit, "pvSite")?;
    insert::insertar_objeto(
        prj,
        "pvProject",
        &bloque,
        Some("SitePrj"),
        insert::SiExiste::Reemplazar,
    )
}
