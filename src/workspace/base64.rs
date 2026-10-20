//! Base64 encoding and decoding for binary workspace files.

use crate::error::{Error, Result};

const TABLA_BASE64: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";

pub(crate) fn base64(data: &[u8]) -> String {
    let mut out = String::with_capacity(data.len().div_ceil(3) * 4);
    for chunk in data.chunks(3) {
        let b = [
            chunk[0],
            *chunk.get(1).unwrap_or(&0),
            *chunk.get(2).unwrap_or(&0),
        ];
        let n = ((b[0] as u32) << 16) | ((b[1] as u32) << 8) | b[2] as u32;
        out.push(TABLA_BASE64[((n >> 18) & 63) as usize] as char);
        out.push(TABLA_BASE64[((n >> 12) & 63) as usize] as char);
        out.push(if chunk.len() > 1 {
            TABLA_BASE64[((n >> 6) & 63) as usize] as char
        } else {
            '='
        });
        out.push(if chunk.len() > 2 {
            TABLA_BASE64[(n & 63) as usize] as char
        } else {
            '='
        });
    }
    out
}

/// Value of one symbol of the standard alphabet (RFC 4648 §4: A–Z, a–z, 0–9, `+`, `/`).
fn valor_simbolo(c: u8) -> Option<u8> {
    match c {
        b'A'..=b'Z' => Some(c - b'A'),
        b'a'..=b'z' => Some(c - b'a' + 26),
        b'0'..=b'9' => Some(c - b'0' + 52),
        b'+' => Some(62),
        b'/' => Some(63),
        _ => None,
    }
}

/// Decode canonical standard Base64. CR/LF between symbols are skipped (MIME
/// line wrapping); otherwise the length must be a multiple of 4, `=` may only
/// appear as one or two trailing pads, and the unused trailing bits must be
/// zero, so every accepted input maps to exactly one byte string. O(n).
pub(crate) fn decode_base64(s: &str) -> Result<Vec<u8>> {
    let invalido = |motivo: String| Error::Invalid(format!("Base64 inválido: {motivo}"));
    let simbolos: Vec<u8> = s.bytes().filter(|c| !matches!(c, b'\r' | b'\n')).collect();
    if !simbolos.len().is_multiple_of(4) {
        return Err(invalido("la longitud no es múltiplo de 4".into()));
    }
    let relleno = simbolos.iter().rev().take_while(|&&c| c == b'=').count();
    if relleno > 2 {
        return Err(invalido("más de dos '=' de relleno".into()));
    }
    let datos = &simbolos[..simbolos.len() - relleno];
    let mut out = Vec::with_capacity(datos.len() / 4 * 3 + 2);
    let (mut buf, mut bits) = (0u32, 0u32);
    for &c in datos {
        let v = valor_simbolo(c)
            .ok_or_else(|| invalido(format!("carácter '{}' no permitido", c as char)))?;
        buf = (buf << 6) | u32::from(v);
        bits += 6;
        if bits >= 8 {
            bits -= 8;
            out.push((buf >> bits) as u8);
        }
    }
    if buf & ((1 << bits) - 1) != 0 {
        return Err(invalido("bits sobrantes distintos de cero".into()));
    }
    Ok(out)
}

#[cfg(test)]
#[path = "../../tests/unit/workspace/base64.rs"]
mod tests;
