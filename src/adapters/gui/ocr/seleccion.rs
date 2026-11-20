//! Picks one form field by the label a person would use for it. Pure: the
//! caller supplies the fields already paired with their labels.

use crate::error::{Error, Result};
use crate::idioma::{Idioma, Texto};

/// A form field with the label found for it (`None` when none was found).
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct CampoEtiquetado {
    pub(crate) hwnd: isize,
    pub(crate) clase: String,
    pub(crate) etiqueta: Option<String>,
    /// Top and left of the field on screen: the order `indice` counts in.
    pub(crate) posicion: (i32, i32),
}

/// What tells apart fields that share a label: their class, and which of the
/// remaining ones (from 0, top to bottom, left to right).
#[derive(Debug, Clone, Copy, Default)]
pub(crate) struct Desambiguador<'a> {
    pub(crate) clase: Option<&'a str>,
    pub(crate) indice: Option<usize>,
}

const VARIOS: Texto = Texto {
    es: "'{buscada}' coincide con varios campos: {etiquetas}. Indica 'indice' (desde 0, de arriba abajo) o 'clase' para elegir uno",
    en: "'{buscada}' matches several fields: {etiquetas}. Pass 'indice' (from 0, top to bottom) or 'clase' to pick one",
};
const FUERA_DE_RANGO: Texto = Texto {
    es: "'indice' {indice} fuera de rango: '{buscada}' coincide con {total} campos",
    en: "'indice' {indice} is out of range: '{buscada}' matches {total} fields",
};

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

/// The fields whose label equals `buscada` (already in [`clave`] form), else
/// those whose label contains it.
fn coincidentes<'a>(campos: &'a [CampoEtiquetado], buscada: &str) -> Vec<&'a CampoEtiquetado> {
    let con_etiqueta = |c: &&CampoEtiquetado| c.etiqueta.is_some();
    let exactos: Vec<&CampoEtiquetado> = campos
        .iter()
        .filter(con_etiqueta)
        .filter(|c| c.etiqueta.as_deref().map(clave).as_deref() == Some(buscada))
        .collect();
    if !exactos.is_empty() {
        return exactos;
    }
    campos
        .iter()
        .filter(con_etiqueta)
        .filter(|c| {
            c.etiqueta
                .as_deref()
                .is_some_and(|e| clave(e).contains(buscada))
        })
        .collect()
}

/// The one field whose label equals `buscada`, or else the one whose label
/// contains it. Several matches are an error listing them, unless `como` picks
/// one: a wrong guess would write into the wrong parameter. O(n log n).
pub(crate) fn campo_por_etiqueta<'a>(
    campos: &'a [CampoEtiquetado],
    buscada: &str,
    como: Desambiguador<'_>,
) -> Result<&'a CampoEtiquetado> {
    let buscada = clave(buscada);
    let candidatos = coincidentes(campos, &buscada);
    let mut candidatos: Vec<&CampoEtiquetado> = candidatos
        .into_iter()
        .filter(|c| como.clase.is_none_or(|clase| c.clase == clase))
        .collect();
    candidatos.sort_by_key(|c| c.posicion);
    if let Some(indice) = como.indice {
        let (i, total) = (indice.to_string(), candidatos.len().to_string());
        return candidatos.get(indice).copied().ok_or_else(|| {
            let vars = [
                ("indice", i.as_str()),
                ("buscada", &buscada),
                ("total", &total),
            ];
            Error::Invalid(FUERA_DE_RANGO.texto_con(Idioma::de_entorno(), &vars))
        });
    }
    match candidatos.as_slice() {
        [unico] => Ok(unico),
        [] => Err(Error::NotFound(format!(
            "No field is called '{buscada}'. Labels seen: {}",
            etiquetas(campos)
        ))),
        varios => {
            let lista = varios
                .iter()
                .filter_map(|c| c.etiqueta.as_deref())
                .collect::<Vec<_>>()
                .join("; ");
            let vars = [("buscada", buscada.as_str()), ("etiquetas", lista.as_str())];
            Err(Error::Invalid(
                VARIOS.texto_con(Idioma::de_entorno(), &vars),
            ))
        }
    }
}

/// Reads a yes/no value as a person writes it.
pub(crate) fn valor_logico(texto: &str) -> Result<bool> {
    match texto.trim().to_lowercase().as_str() {
        "true" | "1" | "si" | "sí" | "yes" | "marcado" | "on" => Ok(true),
        "false" | "0" | "no" | "desmarcado" | "off" => Ok(false),
        otro => Err(Error::Invalid(format!(
            "'{otro}' is not a logical value (true/false, sí/no, 1/0)"
        ))),
    }
}

#[cfg(test)]
#[path = "../../../../tests/unit/adapters/gui/ocr/seleccion.rs"]
mod tests;
