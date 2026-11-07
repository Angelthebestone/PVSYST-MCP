//! The site object of a project: the `pvSite` of a `.SIT` file becomes the project's
//! `SitePrj` and the site inside the variant's `pvMeteo`. Shared by `proyecto crear`
//! and `disenar_sistema`.
//!
//! Both objects must be replaced: the project's `SitePrj` is what the GUI shows, but a
//! simulation takes the site from the variant, so leaving it behind simulates the
//! template's location (Geneva in the demo projects) under the new site's name.

use crate::error::Result;
use crate::format::{edit, insert};
use std::path::Path;

/// Roles PVsyst gives the site object inside a variant's `pvMeteo`: 8.1.8 writes
/// `SiteSimul`, older files `SiteMet`.
const ROLES_SITIO_VARIANTE: [&str; 2] = ["SiteSimul", "SiteMet"];

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

/// `var` with the site object of its `pvMeteo` replaced by the `pvSite` of `sit_file`.
///
/// The role the file already uses is kept, so PVsyst finds the object where it expects it.
pub(crate) fn insertar_sitio_en_variante(var: &[u8], sit_file: &Path) -> Result<Vec<u8>> {
    let sit = crate::workspace::leer_bytes(sit_file)?;
    let bloque = insert::extraer_objeto(&sit, "pvSite")?;
    let rol = rol_del_sitio(var).unwrap_or(ROLES_SITIO_VARIANTE[0]);
    insert::insertar_objeto(
        var,
        "pvMeteo",
        &bloque,
        Some(rol),
        insert::SiExiste::Reemplazar,
    )
}

/// Role the variant already uses for the site of its `pvMeteo`, if it has one.
fn rol_del_sitio(var: &[u8]) -> Option<&'static str> {
    let spans = edit::spans_of(var);
    let meteo = spans
        .iter()
        .find(|s| s.ty.eq_ignore_ascii_case("pvMeteo"))?;
    spans
        .iter()
        .filter(|s| {
            s.open > meteo.open && s.close < meteo.close && s.ty.eq_ignore_ascii_case("pvSite")
        })
        .filter_map(|s| s.role.as_deref())
        .find_map(|rol| {
            ROLES_SITIO_VARIANTE
                .into_iter()
                .find(|r| r.eq_ignore_ascii_case(rol))
        })
}

/// `MeteoFileName` of the project, the weather file a simulation reads unless told otherwise.
pub(crate) fn meteo_del_proyecto(prj: &[u8]) -> Option<String> {
    let texto = String::from_utf8_lossy(prj);
    let root = crate::format::object::parse_first(&texto)?;
    root.find_deep("MeteoFileName")
        .map(str::trim)
        .filter(|m| !m.is_empty())
        .map(str::to_string)
}
