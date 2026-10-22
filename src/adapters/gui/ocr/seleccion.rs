//! Picks one form field by the label a person would use for it. Pure: the
//! caller supplies the fields already paired with their labels.

use crate::error::{Error, Result};

/// A form field with the label found for it (`None` when none was found).
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct CampoEtiquetado {
    pub(crate) hwnd: isize,
    pub(crate) clase: String,
    pub(crate) etiqueta: Option<String>,
}

/// Lower-cases and collapses whitespace so `Factor  de Pérdida` equals
/// `factor de pérdida`.
fn clave(texto: &str) -> String {
    texto
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
        .to_lowercase()
}

fn etiquetas(campos: &[CampoEtiquetado]) -> String {
    campos
        .iter()
        .filter_map(|c| c.etiqueta.as_deref())
        .collect::<Vec<_>>()
        .join("; ")
}

/// The one field whose label equals `buscada`, or else the one whose label
/// contains it. Several matches are an error listing them: a wrong guess would
/// write into the wrong parameter. O(n).
pub(crate) fn campo_por_etiqueta<'a>(
    campos: &'a [CampoEtiquetado],
    buscada: &str,
) -> Result<&'a CampoEtiquetado> {
    let buscada = clave(buscada);
    let con_etiqueta = |c: &&CampoEtiquetado| c.etiqueta.is_some();
    let exactos: Vec<&CampoEtiquetado> = campos
        .iter()
        .filter(con_etiqueta)
        .filter(|c| c.etiqueta.as_deref().map(clave).as_deref() == Some(buscada.as_str()))
        .collect();
    let candidatos = if exactos.is_empty() {
        campos
            .iter()
            .filter(con_etiqueta)
            .filter(|c| {
                c.etiqueta
                    .as_deref()
                    .is_some_and(|e| clave(e).contains(&buscada))
            })
            .collect()
    } else {
        exactos
    };
    match candidatos.as_slice() {
        [unico] => Ok(unico),
        [] => Err(Error::NotFound(format!(
            "Ningún campo se llama '{buscada}'. Rótulos vistos: {}",
            etiquetas(campos)
        ))),
        varios => Err(Error::Invalid(format!(
            "'{buscada}' coincide con varios campos: {}",
            varios
                .iter()
                .filter_map(|c| c.etiqueta.as_deref())
                .collect::<Vec<_>>()
                .join("; ")
        ))),
    }
}

/// Reads a yes/no value as a person writes it.
pub(crate) fn valor_logico(texto: &str) -> Result<bool> {
    match texto.trim().to_lowercase().as_str() {
        "true" | "1" | "si" | "sí" | "yes" | "marcado" | "on" => Ok(true),
        "false" | "0" | "no" | "desmarcado" | "off" => Ok(false),
        otro => Err(Error::Invalid(format!(
            "'{otro}' no es un valor lógico (true/false, sí/no, 1/0)"
        ))),
    }
}

#[cfg(test)]
#[path = "../../../../tests/unit/adapters/gui/ocr/seleccion.rs"]
mod tests;
