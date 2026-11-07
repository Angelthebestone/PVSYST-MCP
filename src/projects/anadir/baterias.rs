//! Battery configuration adjustments on target `pvSystem` objects.

use crate::error::{Error, Result};
use crate::format::edit;

/// Sets battery configuration keys on the target variant's `pvSystem` object.
pub(crate) fn ajustar_sistema(bytes: &[u8], campos: &[(&str, &str)]) -> Result<Vec<u8>> {
    if edit::find_span(&edit::spans_of(bytes), "pvSystem").is_none() {
        return Err(Error::NotFound(
            "La variante no tiene objeto pvSystem".into(),
        ));
    }
    let cambios: Vec<(String, Option<String>)> = campos
        .iter()
        .map(|(k, v)| ((*k).to_string(), Some((*v).to_string())))
        .collect();
    // `edit_fields` fails on a field it cannot set: no outcome needs checking.
    let (nuevo, _) = edit::edit_fields(bytes, "pvSystem", &cambios)?;
    Ok(nuevo)
}
