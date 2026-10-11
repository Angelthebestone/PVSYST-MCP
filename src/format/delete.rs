//! Object removal and duplicate field cleanup.
//!
//! Works directly on raw byte buffers, preserving untouched lines, binary blocks,
//! line endings (LF/CRLF) and indentation.

use super::object;
use super::selector::parse_selector;
use super::span::{find_span_by_segs, spans_of, ObjSpan};
use crate::error::{Error, Result};

fn split(bytes: &[u8]) -> Vec<Vec<u8>> {
    bytes.split(|&b| b == b'\n').map(<[u8]>::to_vec).collect()
}

fn join(lines: &[Vec<u8>]) -> Vec<u8> {
    lines.join(&b'\n')
}

fn texto(line: &[u8]) -> String {
    String::from_utf8_lossy(line).trim().to_string()
}

/// Direct field lines (`key=value`) of span `si` as (line index, key, value),
/// skipping nested blocks and structural markers.
fn campos(lineas: &[Vec<u8>], spans: &[ObjSpan], si: usize) -> Vec<(usize, String, String)> {
    let sp = &spans[si];
    let anidados: Vec<(usize, usize)> = spans
        .iter()
        .enumerate()
        .filter(|(j, t)| *j != si && t.open > sp.open && t.open < sp.close)
        .map(|(_, t)| (t.open, t.close))
        .collect();

    let mut out = Vec::new();
    let limit = sp.close.min(lineas.len());
    for (i, linea) in lineas.iter().enumerate().take(limit).skip(sp.open + 1) {
        if anidados.iter().any(|&(a, b)| i >= a && i <= b) {
            continue;
        }
        let t = texto(linea);
        if t.is_empty() || t.starts_with("PVObject_") || t.starts_with("End of") {
            continue;
        }
        if let Some((k, v)) = t.split_once('=') {
            out.push((i, k.trim().to_string(), v.trim().to_string()));
        }
    }
    out
}

/// Remove the object `selector` (header through its `End of PVObject` line).
///
/// Refuses with `Error::Format` if the document structure is unbalanced.
pub(crate) fn eliminar_objeto(destino: &[u8], selector: &str) -> Result<Vec<u8>> {
    object::validar_estructura(destino)?;
    let segs = parse_selector(selector)?;
    let spans = spans_of(destino);
    let si = find_span_by_segs(&spans, &segs)
        .ok_or_else(|| Error::NotFound(format!("Object '{selector}' not found")))?;

    let mut lineas = split(destino);
    let sp = &spans[si];
    lineas.drain(sp.open..=sp.close.min(lineas.len() - 1));
    Ok(join(&lineas))
}

/// Keys that occur more than once directly inside `selector`, with every value in
/// file order. Empty when the selector does not match or is invalid.
pub(crate) fn claves_duplicadas(bytes: &[u8], selector: &str) -> Vec<(String, Vec<String>)> {
    let Ok(segs) = parse_selector(selector) else {
        return Vec::new();
    };
    let spans = spans_of(bytes);
    let Some(si) = find_span_by_segs(&spans, &segs) else {
        return Vec::new();
    };

    let lineas = split(bytes);
    let mut grupos: Vec<(String, Vec<String>)> = Vec::new();
    for (_, clave, valor) in campos(&lineas, &spans, si) {
        match grupos.iter_mut().find(|(k, _)| *k == clave) {
            Some((_, valores)) => valores.push(valor),
            None => grupos.push((clave, vec![valor])),
        }
    }
    grupos.retain(|(_, valores)| valores.len() > 1);
    grupos
}

/// Delete every occurrence of a duplicated key after its first one.
///
/// Refuses with `Error::Format` if the document structure is unbalanced.
pub(crate) fn quitar_duplicados(bytes: &[u8], selector: &str) -> Result<Vec<u8>> {
    object::validar_estructura(bytes)?;
    let segs = parse_selector(selector)?;
    let spans = spans_of(bytes);
    let si = find_span_by_segs(&spans, &segs)
        .ok_or_else(|| Error::NotFound(format!("Object '{selector}' not found")))?;

    let duplicadas: Vec<String> = claves_duplicadas(bytes, selector)
        .into_iter()
        .map(|(k, _)| k)
        .collect();

    let mut lineas = split(bytes);
    let mut vistas: Vec<String> = Vec::new();
    let mut borrar: Vec<usize> = Vec::new();

    for (i, clave, _) in campos(&lineas, &spans, si) {
        if !duplicadas.contains(&clave) {
            continue;
        }
        if vistas.contains(&clave) {
            borrar.push(i);
        } else {
            vistas.push(clave);
        }
    }
    for i in borrar.into_iter().rev() {
        lineas.remove(i);
    }
    Ok(join(&lineas))
}

#[cfg(test)]
mod tests {
    use super::*;

    const SINTETICO: &[u8] = b"PVObject_Root=pvRoot\r\n  FieldTilt=20\r\n  FieldTilt=25\r\n  Name=x\r\nEnd of PVObject pvRoot\r\n";

    #[test]
    fn should_detect_and_remove_duplicate_keys() {
        let dup = claves_duplicadas(SINTETICO, "pvRoot");
        assert_eq!(
            dup,
            vec![(
                "FieldTilt".to_string(),
                vec!["20".to_string(), "25".to_string()]
            )]
        );
        let limpio = quitar_duplicados(SINTETICO, "pvRoot").expect("quitar");
        let texto = String::from_utf8(limpio).expect("utf8");
        assert_eq!(texto.matches("FieldTilt=").count(), 1);
        assert!(texto.contains("FieldTilt=20"));
        assert!(texto.contains("Name=x"));
        assert!(texto.ends_with("\r\n"));
    }

    #[test]
    fn should_eliminate_object() {
        let doc = b"PVObject_Root=pvRoot\r\n  PVObject_Hijo=pvChild\r\n    X=1\r\n  End of PVObject pvChild\r\nEnd of PVObject pvRoot\r\n";
        let sin = eliminar_objeto(doc, "pvChild").expect("eliminar");
        let txt = String::from_utf8(sin).expect("utf8");
        assert!(!txt.contains("pvChild"));
        assert!(txt.contains("pvRoot"));
    }

    #[test]
    fn should_refuse_delete_when_unbalanced() {
        let trunc = b"PVObject_Root=pvRoot\r\n  PVObject_Hijo=pvChild\r\n";
        let err = eliminar_objeto(trunc, "pvChild").unwrap_err();
        assert!(matches!(err, Error::Format(_)));
        assert!(err.to_string().contains("sin cerrar"));
    }

    #[test]
    fn should_refuse_delete_when_selector_invalid() {
        let err = eliminar_objeto(SINTETICO, "pvRoot[abc]").unwrap_err();
        assert!(matches!(err, Error::Invalid(_)));
    }

    #[test]
    fn should_refuse_quitar_duplicados_when_unbalanced() {
        let trunc = b"PVObject_Root=pvRoot\r\n  FieldTilt=20\r\n  FieldTilt=25\r\n";
        let err = quitar_duplicados(trunc, "pvRoot").unwrap_err();
        assert!(matches!(err, Error::Format(_)));
    }
}
