//! Base64 encoder for PNG payloads in MCP tool responses.

const BASE64_ALPHABET: &[u8; 64] =
    b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
const PAD_CHAR: char = '=';

fn char_alfabeto(indice: usize) -> char {
    BASE64_ALPHABET
        .get(indice & 63)
        .copied()
        .map(char::from)
        .unwrap_or(PAD_CHAR)
}

fn codificar_trozo(trozo: &[u8], out: &mut String) {
    let Some(&b0) = trozo.first() else { return };
    let b1 = trozo.get(1).copied().unwrap_or(0);
    let b2 = trozo.get(2).copied().unwrap_or(0);
    let n = (u32::from(b0) << 16) | (u32::from(b1) << 8) | u32::from(b2);
    for i in 0..4 {
        if i <= trozo.len() {
            out.push(char_alfabeto(((n >> (18 - 6 * i)) & 63) as usize));
        } else {
            out.push(PAD_CHAR);
        }
    }
}

/// Standard base64 with padding.
pub(crate) fn base64(datos: &[u8]) -> String {
    let mut out = String::with_capacity(datos.len().div_ceil(3) * 4);
    for trozo in datos.chunks(3) {
        codificar_trozo(trozo, &mut out);
    }
    out
}

#[cfg(test)]
#[path = "../../../tests/unit/adapters/gui/base64.rs"]
mod tests;
