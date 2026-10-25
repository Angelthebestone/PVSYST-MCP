//! `Comment` of project objects: the explicit `comentario` argument and the DEMO
//! comment rule shared by project creation and the doctor.

use crate::error::Result;
use crate::format::{edit, object};
use serde_json::Value;

/// Replaces the first object's DEMO `Comment` with `nombre`. `None` when there is no DEMO comment.
pub(crate) fn sustituir_demo(
    bytes: &[u8],
    nombre: &str,
) -> Result<Option<(Vec<u8>, Vec<edit::ChangeOutcome>)>> {
    let texto = String::from_utf8_lossy(bytes);
    let demo = object::parse_first(&texto)
        .and_then(|raiz| raiz.field("Comment").map(|c| c.contains("DEMO")))
        .unwrap_or(false);
    let ty = edit::spans_of(bytes).first().map(|s| s.ty.clone());
    match ty {
        Some(ty) if demo && !nombre.starts_with("_DEMO") => {
            let cambios = [("Comment".to_string(), Some(nombre.to_string()))];
            Ok(Some(edit::edit_fields(bytes, &ty, &cambios)?))
        }
        _ => Ok(None),
    }
}

/// Comment rule for one new object: the explicit text, or the project name over a DEMO comment.
pub(crate) struct Comentario<'a> {
    pub(crate) explicito: Option<&'a str>,
    pub(crate) nombre: &'a str,
}

impl Comentario<'_> {
    /// Applies the rule to `objeto` (`pvProject` / `pvVCalcul`); returns the bytes and the change report.
    pub(crate) fn aplicar(&self, bytes: &[u8], objeto: &str) -> Result<(Vec<u8>, Vec<Value>)> {
        let cambio = match self.explicito {
            Some(c) => {
                let cambios = [("Comment".to_string(), Some(c.to_string()))];
                Some(edit::edit_fields(bytes, objeto, &cambios)?)
            }
            None => sustituir_demo(bytes, self.nombre)?,
        };
        Ok(match cambio {
            Some((nuevo, cambios)) => (
                nuevo,
                cambios.iter().map(edit::ChangeOutcome::to_json).collect(),
            ),
            None => (bytes.to_vec(), Vec::new()),
        })
    }
}
