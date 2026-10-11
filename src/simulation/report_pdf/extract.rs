use miniz_oxide::inflate::decompress_to_vec_zlib;

/// Text-showing strings of every inflatable content stream, in file order.
pub(crate) fn tokens(pdf: &[u8]) -> Vec<String> {
    let mut out = Vec::new();
    for (start, end) in stream_bodies(pdf) {
        let body = &pdf[start..end];
        let inflated = decompress_to_vec_zlib(body).or_else(|_| {
            // Some writers leave the EOL before `endstream` inside the body.
            let trimmed = body.strip_suffix(b"\n").unwrap_or(body);
            let trimmed = trimmed.strip_suffix(b"\r").unwrap_or(trimmed);
            decompress_to_vec_zlib(trimmed)
        });
        if let Ok(content) = inflated {
            extract_text(&content, &mut out);
        }
    }
    out
}

/// Byte ranges `(start, end)` of every `stream ... endstream` body.
fn stream_bodies(pdf: &[u8]) -> Vec<(usize, usize)> {
    let mut bodies = Vec::new();
    let mut pos = 0;
    while let Some(rel) = find(&pdf[pos..], b"stream") {
        let at = pos + rel;
        pos = at + b"stream".len();
        if at >= 3 && &pdf[at - 3..at] == b"end" {
            continue;
        }
        let mut start = pos;
        if pdf.get(start) == Some(&b'\r') {
            start += 1;
        }
        if pdf.get(start) == Some(&b'\n') {
            start += 1;
        } else {
            continue;
        }
        let Some(end_rel) = find(&pdf[start..], b"endstream") else {
            break;
        };
        bodies.push((start, start + end_rel));
        pos = start + end_rel;
    }
    bodies
}

fn find(haystack: &[u8], needle: &[u8]) -> Option<usize> {
    haystack.windows(needle.len()).position(|w| w == needle)
}

/// Collect `(...)Tj` strings and `[...]TJ` arrays (joined) from a content stream.
fn extract_text(content: &[u8], out: &mut Vec<String>) {
    let mut i = 0;
    while i < content.len() {
        match content[i] {
            b'(' => match literal(content, i) {
                Some((bytes, end)) => {
                    if followed_by(content, end, b"Tj") {
                        out.push(latin1(&bytes));
                    }
                    i = end;
                }
                None => i += 1,
            },
            b'[' => {
                let (parts, end) = array(content, i);
                if followed_by(content, end, b"TJ") && !parts.is_empty() {
                    out.push(latin1(&parts.concat()));
                    i = end;
                } else {
                    i += 1;
                }
            }
            _ => i += 1,
        }
    }
}

/// Parse a PDF literal string starting at `i` (which must be `(`). Returns the
/// unescaped bytes and the index just past the closing parenthesis.
fn literal(s: &[u8], i: usize) -> Option<(Vec<u8>, usize)> {
    let mut buf = Vec::new();
    let mut j = i + 1;
    while j < s.len() {
        match s[j] {
            b'\\' if j + 1 < s.len() => {
                buf.push(s[j + 1]);
                j += 2;
            }
            b')' => return Some((buf, j + 1)),
            b => {
                buf.push(b);
                j += 1;
            }
        }
    }
    None
}

/// Literal strings inside a `[...]` array starting at `i`; returns them and
/// the index just past `]` (or the end of the stream when unterminated).
fn array(s: &[u8], i: usize) -> (Vec<Vec<u8>>, usize) {
    let mut parts = Vec::new();
    let mut j = i + 1;
    while j < s.len() {
        match s[j] {
            b'(' => match literal(s, j) {
                Some((bytes, end)) => {
                    parts.push(bytes);
                    j = end;
                }
                None => j += 1,
            },
            b']' => return (parts, j + 1),
            _ => j += 1,
        }
    }
    (parts, s.len())
}

/// True when the next non-whitespace bytes after `i` are `op`.
fn followed_by(s: &[u8], i: usize, op: &[u8]) -> bool {
    let k = s[i..]
        .iter()
        .position(|b| !b.is_ascii_whitespace())
        .map_or(s.len(), |p| i + p);
    s[k..].starts_with(op)
}

fn latin1(bytes: &[u8]) -> String {
    bytes.iter().map(|&b| char::from(b)).collect()
}
