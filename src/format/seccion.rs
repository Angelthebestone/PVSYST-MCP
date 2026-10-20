//! Edition of the plain `Clave=TType … End of TType` sections PVsyst uses for
//! settings that are not `PVObject_` trees (`EconomicEval`, `WaterUse`).
//!
//! Only direct fields (opening indentation + 2) are reachable: nested lists and
//! `PVObject_` blocks keep their own editors.

use super::edit::{field_line, outcome, reject_line_breaks, ChangeOutcome, FinDeLinea};
use super::span::{indent_width, tstart, Lines};
use crate::error::{Error, Result};

/// Direct fields sit two spaces deeper than the section header.
const INDENT_CAMPO: usize = 2;

struct Seccion {
    apertura: usize,
    cierre: usize,
    indent: usize,
}

fn cabecera(linea: &str, clave: &str) -> Option<String> {
    let tipo = tstart(linea).strip_prefix(clave)?.strip_prefix('=')?.trim();
    tipo.starts_with('T').then(|| tipo.to_string())
}

/// PVsyst closes sections either with the declared type (`End of TEcoEvaluation`)
/// or with the short name (`End of WaterUse`); both forms are accepted.
fn cierra(linea: &str, tipo: &str, clave: &str) -> bool {
    let Some(nombre) = tstart(linea).trim_end().strip_prefix("End of ") else {
        return false;
    };
    nombre.eq_ignore_ascii_case(tipo) || nombre.eq_ignore_ascii_case(clave)
}

fn buscar_seccion(doc: &Lines, clave: &str) -> Option<Seccion> {
    let apertura = (0..doc.lines.len()).find(|&i| cabecera(&doc.text(i), clave).is_some())?;
    let tipo = cabecera(&doc.text(apertura), clave)?;
    let indent = indent_width(&doc.lines[apertura]);
    let cierre = ((apertura + 1)..doc.lines.len())
        .find(|&i| indent_width(&doc.lines[i]) == indent && cierra(&doc.text(i), &tipo, clave))?;
    Some(Seccion {
        apertura,
        cierre,
        indent,
    })
}

fn es_campo_directo(doc: &Lines, seccion: &Seccion, i: usize) -> bool {
    if indent_width(&doc.lines[i]) != seccion.indent + INDENT_CAMPO {
        return false;
    }
    tstart(&doc.text(i)).contains('=')
}

fn campo_de(doc: &Lines, i: usize, clave: &str) -> Option<String> {
    let texto = doc.text(i);
    let (k, v) = tstart(&texto).split_once('=')?;
    k.trim()
        .eq_ignore_ascii_case(clave)
        .then(|| v.trim().to_string())
}

fn indices_de_campo(doc: &Lines, seccion: &Seccion, clave: &str) -> Vec<(usize, String)> {
    ((seccion.apertura + 1)..seccion.cierre)
        .filter(|&i| es_campo_directo(doc, seccion, i))
        .filter_map(|i| Some((i, campo_de(doc, i, clave)?)))
        .collect()
}

fn fin_de(linea: &[u8], defecto: FinDeLinea) -> FinDeLinea {
    if linea.ends_with(b"\r") {
        FinDeLinea::Crlf
    } else {
        defecto
    }
}

fn aplicar_cambio(
    doc: &mut Lines,
    indices: &[(usize, String)],
    clave: &str,
    valor: Option<&str>,
    defecto: FinDeLinea,
) -> ChangeOutcome {
    let Some((_, viejo)) = indices.first() else {
        return match valor {
            Some(nuevo) => outcome(clave, "inserted", None, Some(nuevo)),
            None => outcome(clave, "not-found", None, None),
        };
    };
    match valor {
        Some(nuevo) => {
            for (i, _) in indices {
                let fin = fin_de(&doc.lines[*i], defecto);
                doc.lines[*i] = field_line(indent_width(&doc.lines[*i]), clave, nuevo, fin);
            }
            outcome(clave, "replaced", Some(viejo.clone()), Some(nuevo))
        }
        None => {
            for (i, _) in indices {
                doc.lines[*i].clear();
                doc.newline[*i] = false;
            }
            outcome(clave, "deleted", Some(viejo.clone()), None)
        }
    }
}

/// Apply `cambios` to every direct occurrence of each key inside `clave`.
///
/// Replacing touches all duplicated lines (PVsyst repeats `DiscountRate`);
/// deleting removes them; a missing key is inserted before the closing marker.
pub(crate) fn editar_campos_seccion(
    bytes: &[u8],
    clave: &str,
    cambios: &[(String, Option<String>)],
) -> Result<(Vec<u8>, Vec<ChangeOutcome>)> {
    reject_line_breaks(cambios)?;
    let mut doc = Lines::split(bytes);
    let seccion = buscar_seccion(&doc, clave)
        .ok_or_else(|| Error::NotFound(format!("Sección '{clave}' no encontrada")))?;
    let defecto = fin_de(&doc.lines[seccion.apertura], FinDeLinea::Lf);
    let inserts: Vec<(String, Option<String>)> = cambios
        .iter()
        .filter(|(k, _)| indices_de_campo(&doc, &seccion, k).is_empty())
        .filter(|(_, v)| v.is_some())
        .cloned()
        .collect();

    // WHY a mutable accumulator: the document is edited in place, and the
    // borrow of `doc` cannot be carried through a purely functional fold.
    let outcomes = cambios
        .iter()
        .map(|(k, v)| {
            let indices = indices_de_campo(&doc, &seccion, k);
            aplicar_cambio(&mut doc, &indices, k, v.as_deref(), defecto)
        })
        .collect();

    for (k, v) in inserts.iter().rev() {
        if let Some(valor) = v {
            let indent = seccion.indent + INDENT_CAMPO;
            doc.lines
                .insert(seccion.cierre, field_line(indent, k, valor, defecto));
            doc.newline.insert(seccion.cierre, true);
        }
    }

    Ok((doc.join(), outcomes))
}

/// Whether `bytes` declares the section `clave`.
pub(crate) fn existe_seccion(bytes: &[u8], clave: &str) -> bool {
    buscar_seccion(&Lines::split(bytes), clave).is_some()
}

/// Value of the first direct field `campo` inside section `clave`.
pub(crate) fn campo_seccion(bytes: &[u8], clave: &str, campo: &str) -> Option<String> {
    let doc = Lines::split(bytes);
    let seccion = buscar_seccion(&doc, clave)?;
    ((seccion.apertura + 1)..seccion.cierre)
        .filter(|&i| es_campo_directo(&doc, &seccion, i))
        .find_map(|i| campo_de(&doc, i, campo))
}

#[cfg(test)]
#[path = "../../tests/unit/format/seccion.rs"]
mod tests;
