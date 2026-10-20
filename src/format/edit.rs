//! Line- and byte-preserving editor for PVsyst `PVObject` text files.
//!
//! Works directly on raw bytes so untouched lines, including legacy binary blocks,
//! remain strictly byte-identical.

pub(crate) use super::span::{find_span, spans_of, ObjSpan};

use super::selector::parse_selector;
use super::span::{find_span_by_segs, indent_width, Lines};

use super::object;
use super::span::tstart;
use super::value_text;
use crate::error::{Error, Result};
use serde_json::{json, Map, Value};

/// Line terminator to append to a generated field line.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum FinDeLinea {
    Lf,
    Crlf,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ChangeOutcome {
    pub key: String,
    pub action: String, // "replaced" | "inserted" | "deleted" | "not-found"
    pub old_value: Option<String>,
    pub new_value: Option<String>,
}

/// One object selector with the field changes to apply to it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct EdicionObjeto {
    pub(crate) selector: String,
    pub(crate) cambios: Vec<(String, Option<String>)>,
}

impl ChangeOutcome {
    pub(crate) fn to_json(&self) -> Value {
        json!({
            "key": self.key, "action": self.action,
            "old": self.old_value, "new": self.new_value
        })
    }
}

/// JSON `{campo: valor}` as edit changes: `null` deletes the field.
pub(crate) fn changes_from_json(campos: &Map<String, Value>) -> Vec<(String, Option<String>)> {
    campos
        .iter()
        .map(|(k, v)| (k.clone(), (!v.is_null()).then(|| value_text(v))))
        .collect()
}

pub(super) fn outcome(
    key: &str,
    action: &str,
    old: Option<String>,
    new: Option<&str>,
) -> ChangeOutcome {
    ChangeOutcome {
        key: key.to_string(),
        action: action.to_string(),
        old_value: old,
        new_value: new.map(str::to_string),
    }
}

pub(super) fn reject_line_breaks(changes: &[(String, Option<String>)]) -> Result<()> {
    let breaks = |s: &str| s.contains('\r') || s.contains('\n');
    match changes
        .iter()
        .find(|(key, value)| breaks(key) || value.as_deref().is_some_and(breaks))
    {
        Some((key, _)) => Err(Error::Invalid(format!(
            "Valor con salto de línea en '{key}'"
        ))),
        None => Ok(()),
    }
}

fn is_marker(trimmed: &str) -> bool {
    trimmed.starts_with("PVObject_")
        || trimmed.starts_with("End of")
        || trimmed == "/"
        || trimmed == "EndTags"
        || trimmed.starts_with("End Remarks")
}

fn field_indent(lines: &Lines, span: &ObjSpan) -> usize {
    let limit = span.close.min(lines.lines.len().saturating_sub(1));
    for i in (span.open + 1)..=limit {
        let t = lines.text(i);
        let trimmed = tstart(&t);
        if trimmed.is_empty() || is_marker(trimmed) {
            continue;
        }
        if trimmed.contains('=') {
            let ind = indent_width(&lines.lines[i]);
            if ind > span.indent {
                return ind;
            }
        }
    }
    span.indent + 2
}

fn find_field_line(
    doc: &Lines,
    span: &ObjSpan,
    in_child: &dyn Fn(usize) -> bool,
    key: &str,
) -> Option<(usize, String)> {
    let limit = span.close.min(doc.lines.len().saturating_sub(1));
    for i in (span.open + 1)..=limit {
        let t = doc.text(i);
        let trimmed = tstart(&t);
        if in_child(i) || is_marker(trimmed) {
            continue;
        }
        if let Some((k, v)) = trimmed.split_once('=') {
            if k.trim().eq_ignore_ascii_case(key) {
                return Some((i, v.trim().to_string()));
            }
        }
    }
    None
}

pub(crate) fn field_line(indent: usize, key: &str, value: &str, fin: FinDeLinea) -> Vec<u8> {
    let mut line = vec![b' '; indent];
    line.extend_from_slice(key.as_bytes());
    line.push(b'=');
    line.extend_from_slice(value.as_bytes());
    if matches!(fin, FinDeLinea::Crlf) {
        line.push(b'\r');
    }
    line
}

fn apply_field_change(
    doc: &mut Lines,
    span: &ObjSpan,
    in_child: &dyn Fn(usize) -> bool,
    key: &str,
    value: Option<&str>,
) -> Result<ChangeOutcome> {
    let found = find_field_line(doc, span, in_child, key);
    match (found, value) {
        (Some((i, old)), Some(new)) => {
            let fin = if doc.lines[i].ends_with(b"\r") {
                FinDeLinea::Crlf
            } else {
                FinDeLinea::Lf
            };
            let indent = indent_width(&doc.lines[i]);
            doc.lines[i] = field_line(indent, key, new, fin);
            Ok(outcome(key, "replaced", Some(old), Some(new)))
        }
        (Some((i, old)), None) => {
            doc.lines[i].clear();
            doc.newline[i] = false;
            Ok(outcome(key, "deleted", Some(old), None))
        }
        (None, Some(new)) => Ok(outcome(key, "inserted", None, Some(new))),
        (None, None) => Err(Error::NotFound(format!("Campo '{key}' no existe"))),
    }
}

/// Apply `changes` (key -> Some(value) set, None delete) to one object.
///
/// Refuses with `Error::Format` if document structure is unbalanced, and with
/// `Error::Invalid` if the selector or changes are malformed.
pub(crate) fn edit_fields(
    bytes: &[u8],
    selector: &str,
    changes: &[(String, Option<String>)],
) -> Result<(Vec<u8>, Vec<ChangeOutcome>)> {
    object::validar_estructura(bytes)?;
    let segs = parse_selector(selector)?;
    reject_line_breaks(changes)?;

    let mut doc = Lines::split(bytes);
    let spans = spans_of(bytes);
    let idx = find_span_by_segs(&spans, &segs)
        .ok_or_else(|| Error::NotFound(format!("Object '{selector}' not found")))?;
    let span = spans[idx].clone();
    let find = field_indent(&doc, &span);

    let children: Vec<(usize, usize)> = spans
        .iter()
        .filter(|s| s.open > span.open && s.open <= span.close)
        .map(|s| (s.open, s.close))
        .collect();
    let in_child = |i: usize| children.iter().any(|&(o, c)| (o..=c).contains(&i));

    let mut outcomes = Vec::with_capacity(changes.len());
    let mut inserts: Vec<(&str, &str)> = Vec::new();

    for (key, value) in changes {
        let outcome = apply_field_change(&mut doc, &span, &in_child, key, value.as_deref())?;
        if outcome.action == "inserted" {
            if let Some(v) = value.as_deref() {
                inserts.push((key, v));
            }
        }
        outcomes.push(outcome);
    }

    let fin = if doc.lines[span.open].ends_with(b"\r") {
        FinDeLinea::Crlf
    } else {
        FinDeLinea::Lf
    };
    for (key, value) in inserts.into_iter().rev() {
        doc.lines
            .insert(span.open + 1, field_line(find, key, value, fin));
        doc.newline.insert(span.open + 1, true);
    }

    Ok((doc.join(), outcomes))
}

/// Apply a list of object editions in order, accumulating their outcomes.
///
/// Refuses with the same errors as [`edit_fields`]; an edition that fails leaves
/// the bytes of the preceding ones only in memory, never on disk.
pub(crate) fn editar_objetos(
    bytes: &[u8],
    ediciones: &[EdicionObjeto],
) -> Result<(Vec<u8>, Vec<ChangeOutcome>)> {
    ediciones.iter().try_fold(
        (bytes.to_vec(), Vec::new()),
        |(actual, acumulados), edicion| {
            let (nuevo, outcomes) = edit_fields(&actual, &edicion.selector, &edicion.cambios)?;
            Ok((nuevo, acumulados.into_iter().chain(outcomes).collect()))
        },
    )
}

#[cfg(test)]
#[path = "../../tests/unit/format/edit_tests.rs"]
mod tests;
