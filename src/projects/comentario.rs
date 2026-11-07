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
    ///
    /// Without an explicit comment the object takes the new project's name, whatever the
    /// template said: a copy named `Z_New` must not keep announcing the template it came
    /// from, and the answer then reports the change instead of an empty `applied`.
    pub(crate) fn aplicar(&self, bytes: &[u8], objeto: &str) -> Result<(Vec<u8>, Vec<Value>)> {
        let texto = self.explicito.unwrap_or(self.nombre);
        let cambios = [("Comment".to_string(), Some(texto.to_string()))];
        let (nuevo, outcomes) = edit::edit_fields(bytes, objeto, &cambios)?;
        let aplicados = outcomes
            .iter()
            .filter(|o| o.action != "sin_cambio")
            .map(edit::ChangeOutcome::to_json)
            .collect();
        Ok((nuevo, aplicados))
    }
}
