//! Parser and validator for PVObject selectors.
//!
//! Selectors navigate the PVObject tree by object type or role, with an optional
//! zero-based index to distinguish sibling instances (e.g. `pvVCalcul/pvSubArray[1]`).
//! Role prefixes like `role:SitePrj` match against the object's role rather than its type.

use crate::error::{Error, Result};
use std::fmt;

/// A single segment in a selector path (e.g. `pvSubArray[1]`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct Seg {
    pub(crate) name: String,
    pub(crate) index: usize,
}

impl fmt::Display for Seg {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}[{}]", self.name, self.index)
    }
}

impl Seg {
    /// True when this segment matches the given object type or optional role.
    pub(crate) fn matches(&self, ty: &str, role: Option<&str>) -> bool {
        ty.eq_ignore_ascii_case(&self.name)
            || role.is_some_and(|r| r.eq_ignore_ascii_case(&self.name))
    }
}

/// Strip optional `role:` prefix and trim leading/trailing whitespace.
fn clean_name(raw: &str) -> &str {
    raw.strip_prefix("role:").unwrap_or(raw).trim()
}

/// Parse and validate the index portion inside brackets: `[<index>]`.
fn parse_index(after_open: &str, selector: &str) -> Result<usize> {
    if !after_open.ends_with(']') {
        let msg = if after_open.contains(']') {
            format!("Texto tras corchete de cierre ']' en selector '{selector}'")
        } else {
            format!("Falta corchete de cierre ']' en selector '{selector}'")
        };
        return Err(Error::Invalid(msg));
    }

    let inner = &after_open[..after_open.len() - 1];
    if inner.contains('[') || inner.contains(']') {
        return Err(Error::Invalid(format!(
            "Corchetes anidados o extras en selector '{selector}'"
        )));
    }

    let inner_trimmed = inner.trim();
    if inner_trimmed.is_empty() {
        return Err(Error::Invalid(format!(
            "Índice vacío entre corchetes en selector '{selector}'"
        )));
    }

    inner_trimmed.parse::<usize>().map_err(|_| {
        Error::Invalid(format!(
            "Índice no numérico '{inner_trimmed}' en selector '{selector}'"
        ))
    })
}

/// Parse and validate a single segment (e.g. `pvSubArray[1]`, `role:SitePrj`).
fn parse_segment(part: &str, selector: &str) -> Result<Seg> {
    let trimmed = part.trim();
    if trimmed.is_empty() {
        return Err(Error::Invalid(format!(
            "Segmento vacío en selector '{selector}'"
        )));
    }

    if let Some((name_raw, after_open)) = trimmed.split_once('[') {
        let index = parse_index(after_open, selector)?;
        let name = clean_name(name_raw);
        if name.is_empty() {
            return Err(Error::Invalid(format!(
                "Nombre de objeto vacío en selector '{selector}'"
            )));
        }
        Ok(Seg {
            name: name.to_string(),
            index,
        })
    } else {
        if trimmed.contains(']') {
            return Err(Error::Invalid(format!(
                "Corchete de cierre ']' sin apertura en selector '{selector}'"
            )));
        }
        let name = clean_name(trimmed);
        if name.is_empty() {
            return Err(Error::Invalid(format!(
                "Nombre de objeto vacío en selector '{selector}'"
            )));
        }
        Ok(Seg {
            name: name.to_string(),
            index: 0,
        })
    }
}

/// Parse a selector path like `pvVCalcul/pvSubArray[1]` or `role:SitePrj` into segments.
///
/// Rejects malformed selectors (empty segments, empty names, missing or extra brackets,
/// non-integer or empty indices) with `Error::Invalid` naming the selector.
pub(crate) fn parse_selector(selector: &str) -> Result<Vec<Seg>> {
    let trimmed = selector.trim();
    if trimmed.is_empty() {
        return Err(Error::Invalid(format!("Selector vacío: '{selector}'")));
    }

    let mut segs = Vec::new();
    for part in selector.split('/') {
        segs.push(parse_segment(part, selector)?);
    }
    Ok(segs)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn should_parse_simple_name_with_default_index_zero() {
        let segs = parse_selector("pvProject").expect("valid selector");
        assert_eq!(
            segs,
            vec![Seg {
                name: "pvProject".into(),
                index: 0
            }]
        );
    }

    #[test]
    fn should_parse_indexed_segment() {
        let segs = parse_selector("pvSubArray[1]").expect("valid indexed selector");
        assert_eq!(
            segs,
            vec![Seg {
                name: "pvSubArray".into(),
                index: 1
            }]
        );
    }

    #[test]
    fn should_parse_role_prefix() {
        let segs = parse_selector("role:SitePrj").expect("valid role selector");
        assert_eq!(
            segs,
            vec![Seg {
                name: "SitePrj".into(),
                index: 0
            }]
        );
    }

    #[test]
    fn should_parse_multi_segment_path() {
        let segs = parse_selector("pvVCalcul/pvSubArray[2]").expect("valid multi-segment");
        assert_eq!(
            segs,
            vec![
                Seg {
                    name: "pvVCalcul".into(),
                    index: 0
                },
                Seg {
                    name: "pvSubArray".into(),
                    index: 2
                },
            ]
        );
    }

    #[test]
    fn should_reject_empty_selector() {
        let err = parse_selector("").unwrap_err();
        assert!(matches!(err, Error::Invalid(_)));
        assert!(err.to_string().contains("''"));
    }

    #[test]
    fn should_reject_empty_segment_when_slashes_are_consecutive() {
        let err = parse_selector("a//b").unwrap_err();
        assert!(matches!(err, Error::Invalid(_)));
        assert!(err.to_string().contains("'a//b'"));
    }

    #[test]
    fn should_reject_leading_or_trailing_slash() {
        assert!(parse_selector("/pvProject").is_err());
        assert!(parse_selector("pvProject/").is_err());
    }

    #[test]
    fn should_reject_empty_name() {
        let err = parse_selector("[0]").unwrap_err();
        assert!(matches!(err, Error::Invalid(_)));
        assert!(err.to_string().contains("'[0]'"));

        let err_role = parse_selector("role:[0]").unwrap_err();
        assert!(matches!(err_role, Error::Invalid(_)));
        assert!(err_role.to_string().contains("'role:[0]'"));
    }

    #[test]
    fn should_reject_missing_closing_bracket() {
        let err = parse_selector("pvSubArray[1").unwrap_err();
        assert!(matches!(err, Error::Invalid(_)));
        assert!(err.to_string().contains("'pvSubArray[1'"));
    }

    #[test]
    fn should_reject_text_after_closing_bracket() {
        let err = parse_selector("pvSubArray[1]abc").unwrap_err();
        assert!(matches!(err, Error::Invalid(_)));
        assert!(err.to_string().contains("'pvSubArray[1]abc'"));
    }

    #[test]
    fn should_reject_non_integer_index() {
        let err = parse_selector("pvSubArray[abc]").unwrap_err();
        assert!(matches!(err, Error::Invalid(_)));
        assert!(err.to_string().contains("'pvSubArray[abc]'"));

        let err_neg = parse_selector("pvSubArray[-1]").unwrap_err();
        assert!(matches!(err_neg, Error::Invalid(_)));
        assert!(err_neg.to_string().contains("'pvSubArray[-1]'"));
    }

    #[test]
    fn should_reject_empty_index() {
        let err = parse_selector("pvSubArray[]").unwrap_err();
        assert!(matches!(err, Error::Invalid(_)));
        assert!(err.to_string().contains("'pvSubArray[]'"));
    }

    #[test]
    fn should_reject_unopened_closing_bracket() {
        let err = parse_selector("pvSubArray]").unwrap_err();
        assert!(matches!(err, Error::Invalid(_)));
        assert!(err.to_string().contains("'pvSubArray]'"));
    }

    #[test]
    fn should_reject_nested_brackets() {
        let err = parse_selector("pvSubArray[1][2]").unwrap_err();
        assert!(matches!(err, Error::Invalid(_)));
        assert!(err.to_string().contains("'pvSubArray[1][2]'"));
    }
}
