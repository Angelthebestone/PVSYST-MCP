//! Byte- and line-level object spans in PVObject documents.
//!
//! Spans identify the exact range of lines (`open..=close`) defining each
//! object block (`PVObject_...` to `End of PVObject`).

use super::cierre::cierra_objeto;
use super::object::Lists;
use super::selector::{parse_selector, Seg};

/// A `PVObject_<Role>=<Type>` … `End of PVObject <Type>` block span.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ObjSpan {
    pub(crate) ty: String,
    pub(crate) role: Option<String>,
    /// Index of the `PVObject_` line.
    pub(crate) open: usize,
    /// Index of the matching `End of PVObject` line (or last line if open).
    pub(crate) close: usize,
    /// Leading whitespace width of the `PVObject_` line.
    pub(crate) indent: usize,
}

impl ObjSpan {
    pub(crate) fn matches(&self, seg: &Seg) -> bool {
        seg.matches(&self.ty, self.role.as_deref())
    }
}

/// Split a byte buffer into lines, keeping original line endings and empty lines.
pub(crate) struct Lines {
    /// Content of each line WITHOUT the trailing `\n` (a trailing `\r` is kept).
    pub(crate) lines: Vec<Vec<u8>>,
    /// Whether each line was terminated by `\n`.
    pub(crate) newline: Vec<bool>,
}

impl Lines {
    pub(crate) fn split(bytes: &[u8]) -> Lines {
        let mut lines = Vec::new();
        let mut newline = Vec::new();
        let mut cur = Vec::new();
        for &b in bytes {
            if b == b'\n' {
                lines.push(std::mem::take(&mut cur));
                newline.push(true);
            } else {
                cur.push(b);
            }
        }
        if !cur.is_empty() {
            lines.push(cur);
            newline.push(false);
        }
        Lines { lines, newline }
    }

    pub(crate) fn join(&self) -> Vec<u8> {
        let mut out = Vec::new();
        for (i, l) in self.lines.iter().enumerate() {
            out.extend_from_slice(l);
            if self.newline.get(i).copied().unwrap_or(false) {
                out.push(b'\n');
            }
        }
        out
    }

    pub(crate) fn text(&self, i: usize) -> String {
        String::from_utf8_lossy(&self.lines[i]).to_string()
    }
}

/// Count leading whitespace characters (spaces or tabs).
pub(crate) fn indent_width(line: &[u8]) -> usize {
    line.iter()
        .take_while(|&&b| b == b' ' || b == b'\t')
        .count()
}

/// Trim leading whitespace and a possible UTF-8 BOM.
pub(crate) fn tstart(s: &str) -> &str {
    s.trim_start_matches(|c: char| c.is_whitespace() || c == '\u{FEFF}')
}

fn split_header(rest: &str) -> (Option<String>, String) {
    match rest.split_once('=') {
        Some((r, ty)) => {
            let r = r.trim();
            (
                if r.is_empty() {
                    None
                } else {
                    Some(r.to_string())
                },
                ty.trim().to_string(),
            )
        }
        None => (None, String::new()),
    }
}

fn unwind_span_stack(stack: &mut Vec<usize>, spans: &mut [ObjSpan], depth: usize, line_idx: usize) {
    while stack.len() > depth {
        if let Some(idx) = stack.pop() {
            spans[idx].close = line_idx;
        }
    }
}

/// Parse all object spans from a line buffer.
pub(crate) fn parse_spans(lines: &Lines) -> Vec<ObjSpan> {
    let mut spans: Vec<ObjSpan> = Vec::new();
    let mut stack: Vec<usize> = Vec::new();
    let mut lists = Lists::new();

    for (i, raw) in lines.lines.iter().enumerate() {
        let t = lines.text(i);
        let trimmed = tstart(&t);

        if let Some(depth) = lists.feed(trimmed, stack.len()) {
            unwind_span_stack(&mut stack, &mut spans, depth, i);
            continue;
        }

        if let Some(rest) = trimmed.strip_prefix("PVObject_") {
            let (role, ty) = split_header(rest);
            let idx = spans.len();
            spans.push(ObjSpan {
                ty,
                role,
                open: i,
                close: i,
                indent: indent_width(raw),
            });
            stack.push(idx);
        } else if let Some(&idx) = stack.last() {
            if cierra_objeto(trimmed, &spans[idx].ty) {
                stack.pop();
                spans[idx].close = i;
            }
        }
    }

    let last = lines.lines.len().saturating_sub(1);
    for idx in stack {
        spans[idx].close = last;
    }
    spans
}

/// Parse all object spans from a byte buffer.
pub(crate) fn spans_of(bytes: &[u8]) -> Vec<ObjSpan> {
    let lines = Lines::split(bytes);
    parse_spans(&lines)
}

/// Locate an object span using already parsed selector segments.
pub(crate) fn find_span_by_segs(spans: &[ObjSpan], segs: &[Seg]) -> Option<usize> {
    if segs.is_empty() {
        return None;
    }
    let mut current: Option<usize> = None;
    for seg in segs {
        let mut hits: Vec<usize> = Vec::new();
        for (i, sp) in spans.iter().enumerate() {
            let in_scope = match current {
                None => true,
                Some(c) => sp.open > spans[c].open && sp.open < spans[c].close,
            };
            if in_scope && sp.matches(seg) {
                hits.push(i);
            }
        }
        let pick = hits.get(seg.index).copied()?;
        current = Some(pick);
    }
    current
}

/// Locate an object span from a selector string.
///
/// Returns `None` if the selector is invalid or no matching object exists.
pub(crate) fn find_span(spans: &[ObjSpan], selector: &str) -> Option<usize> {
    let segs = parse_selector(selector).ok()?;
    find_span_by_segs(spans, &segs)
}

#[cfg(test)]
#[path = "../../tests/unit/format/span.rs"]
mod tests;
