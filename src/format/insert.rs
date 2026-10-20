//! Byte-preserving object insertion and extraction.
//!
//! Works on raw bytes like `format::edit`, preserving untouched lines and
//! adopting destination indentation and line endings.

pub(crate) use super::delete::{claves_duplicadas, eliminar_objeto, quitar_duplicados};

use super::edit;
use super::object;
use super::selector::parse_selector;
use super::span::{find_span_by_segs, ObjSpan};
use crate::error::{Error, Result};

const BOM: &[u8] = &[0xEF, 0xBB, 0xBF];

fn sin_bom(bytes: &[u8]) -> &[u8] {
    bytes.strip_prefix(BOM).unwrap_or(bytes)
}

fn split(bytes: &[u8]) -> Vec<Vec<u8>> {
    bytes.split(|&b| b == b'\n').map(<[u8]>::to_vec).collect()
}

fn join(lines: &[Vec<u8>]) -> Vec<u8> {
    lines.join(&b'\n')
}

fn texto(line: &[u8]) -> String {
    String::from_utf8_lossy(line).trim().to_string()
}

fn same_role(a: Option<&str>, b: Option<&str>) -> bool {
    match (a, b) {
        (Some(x), Some(y)) => x.eq_ignore_ascii_case(y),
        (None, None) => true,
        _ => false,
    }
}

fn indent_of(line: &[u8]) -> usize {
    line.iter()
        .take_while(|&&b| b == b' ' || b == b'\t')
        .count()
}

#[derive(Clone, Copy)]
struct OpcionesBloque<'a> {
    crlf: bool,
    indent: usize,
    rol: Option<&'a str>,
}

fn preparar_bloque(bloque: &[u8], opts: &OpcionesBloque<'_>) -> Result<Vec<Vec<u8>>> {
    let mut lineas = split(sin_bom(bloque));
    if lineas.last().is_some_and(Vec::is_empty) {
        lineas.pop();
    }
    let primera = lineas
        .iter()
        .position(|l| !texto(l).is_empty())
        .ok_or_else(|| Error::Format("El bloque está vacío".into()))?;
    let cabecera = texto(&lineas[primera]);
    let Some(resto) = cabecera.strip_prefix("PVObject_") else {
        return Err(Error::Format(
            "El bloque no empieza por una cabecera PVObject_".into(),
        ));
    };
    let base = indent_of(&lineas[primera]);
    let pad = " ".repeat(opts.indent);
    let mut out = Vec::with_capacity(lineas.len());
    for (i, linea) in lineas.iter().enumerate() {
        let sin_cr = linea.strip_suffix(b"\r").unwrap_or(linea);
        let mut nueva = if texto(sin_cr).is_empty() {
            Vec::new()
        } else {
            let quitar = indent_of(sin_cr).min(base);
            let mut v = pad.as_bytes().to_vec();
            v.extend_from_slice(&sin_cr[quitar..]);
            v
        };
        if i == primera {
            if let Some(nuevo_rol) = opts.rol {
                let ty = resto.split_once('=').map_or("", |(_, t)| t.trim());
                nueva = format!("{pad}PVObject_{nuevo_rol}={ty}").into_bytes();
            }
        }
        if opts.crlf {
            nueva.push(b'\r');
        }
        out.push(nueva);
    }
    Ok(out)
}

/// True when span `si` lies inside span `pi` with no intermediate span between them.
fn es_hijo_directo(spans: &[ObjSpan], pi: usize, si: usize) -> bool {
    let (p, s) = (&spans[pi], &spans[si]);
    !spans
        .iter()
        .any(|t| t.open > p.open && t.open < s.open && t.close > s.close && t.close <= p.close)
}

/// What to do when the parent already holds an object of the inserted type.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum SiExiste {
    /// Fail with an error and leave the file untouched.
    Rechazar,
    /// Replace the existing object.
    Reemplazar,
}

/// Insert `bloque` as a child of the object selected by `selector_padre`.
///
/// Refuses with `Error::Format` if `destino` or `bloque` is unbalanced, and with
/// `Error::Invalid` if `selector_padre` is malformed.
pub(crate) fn insertar_objeto(
    destino: &[u8],
    selector_padre: &str,
    bloque: &[u8],
    rol: Option<&str>,
    si_existe: SiExiste,
) -> Result<Vec<u8>> {
    object::validar_estructura(destino)?;
    object::validar_estructura(bloque)?;
    let segs = parse_selector(selector_padre)?;

    let spans = edit::spans_of(destino);
    let pi = find_span_by_segs(&spans, &segs)
        .ok_or_else(|| Error::NotFound(format!("Object '{selector_padre}' not found")))?;
    let padre = spans[pi].clone();
    let mut lineas = split(destino);
    let crlf = lineas.first().is_some_and(|l| l.ends_with(b"\r"));

    let opts = OpcionesBloque {
        crlf,
        indent: padre.indent + 2,
        rol,
    };
    let nuevo = preparar_bloque(bloque, &opts)?;
    let ns = edit::spans_of(&join(&nuevo))
        .into_iter()
        .next()
        .ok_or_else(|| Error::Format("El bloque no contiene ningún PVObject".into()))?;

    let existente = spans.iter().enumerate().position(|(si, s)| {
        s.open > padre.open
            && s.open < padre.close
            && s.ty.eq_ignore_ascii_case(&ns.ty)
            && same_role(s.role.as_deref(), ns.role.as_deref())
            && es_hijo_directo(&spans, pi, si)
    });
    match existente {
        Some(si) if si_existe == SiExiste::Rechazar => {
            Err(Error::Invalid(format!("Ya existe {}", spans[si].ty)))
        }
        Some(si) => {
            let hijo = &spans[si];
            lineas.splice(hijo.open..=hijo.close, nuevo);
            Ok(join(&lineas))
        }
        None => {
            lineas.splice(padre.close..padre.close, nuevo);
            Ok(join(&lineas))
        }
    }
}

/// Return the lines of the object `selector` from `fuente`, each newline-terminated.
pub(crate) fn extraer_objeto(fuente: &[u8], selector: &str) -> Result<Vec<u8>> {
    let segs = parse_selector(selector)?;
    let spans = edit::spans_of(fuente);
    let si = find_span_by_segs(&spans, &segs)
        .ok_or_else(|| Error::NotFound(format!("Object '{selector}' not found")))?;
    let lineas = split(fuente);
    let sp = &spans[si];
    let mut out = Vec::new();
    let limit = sp.close.min(lineas.len().saturating_sub(1));
    for linea in &lineas[sp.open..=limit] {
        out.extend_from_slice(linea);
        out.push(b'\n');
    }
    Ok(sin_bom(&out).to_vec())
}

#[cfg(test)]
#[path = "../../tests/unit/format/insert.rs"]
mod tests;
