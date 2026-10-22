//! Base62 codec and hourly day line parser/formatter for PVsyst series.
//!
//! Encodes and decodes PVsyst's alphanumeric Base62 alphabet (`0-9`, `A-Z`, `a-z`)
//! used in daily records `Day_ddmm=+val1+val2...`.

use crate::error::{Error, Result};

pub(crate) const BASE_RADIX: u64 = 62;
pub(crate) const TOKENS_POR_DIA: usize = 24;
pub(crate) const LONGITUD_ETIQUETA_DIA: usize = 4;
pub(crate) const PREFIJO_ESPACIOS_DIA: &str = "        ";

/// Converts a character byte to its Base62 digit value (0..61).
pub(crate) fn char_a_valor(c: u8) -> Result<i64> {
    match c {
        b'0'..=b'9' => Ok((c - b'0') as i64),
        b'A'..=b'Z' => Ok((c - b'A' + 10) as i64),
        b'a'..=b'z' => Ok((c - b'a' + 36) as i64),
        _ => Err(Error::Format(format!(
            "Caracter Base62 no reconocido: '{}'",
            c as char
        ))),
    }
}

/// Converts a Base62 digit value (0..61) to its ASCII character byte.
pub(crate) fn valor_a_char(v: u8) -> Result<u8> {
    match v {
        0..=9 => Ok(b'0' + v),
        10..=35 => Ok(b'A' + (v - 10)),
        36..=61 => Ok(b'a' + (v - 36)),
        _ => Err(Error::Format(format!("Valor Base62 fuera de rango: {v}"))),
    }
}

/// Decodes an unsigned Base62 token into a 64-bit integer.
pub(crate) fn decodificar_b62(token: &str) -> Result<i64> {
    if token.is_empty() {
        return Err(Error::Format("Token Base62 vacio".into()));
    }
    token.bytes().try_fold(0i64, |acc, b| {
        let v = char_a_valor(b)?;
        acc.checked_mul(BASE_RADIX as i64)
            .and_then(|m| m.checked_add(v))
            .ok_or_else(|| Error::Format(format!("Desbordamiento decodificando token: '{token}'")))
    })
}

/// Encodes an unsigned 64-bit integer into Base62.
pub(crate) fn codificar_b62(val: u64) -> String {
    if val == 0 {
        return "0".to_string();
    }
    // Local accumulator for base-62 division.
    let mut digitos = Vec::with_capacity(4);
    // Local accumulator for remaining quotient.
    let mut n = val;
    while n > 0 {
        let rem = (n % BASE_RADIX) as u8;
        if let Ok(c) = valor_a_char(rem) {
            digitos.push(c);
        }
        n /= BASE_RADIX;
    }
    digitos.reverse();
    String::from_utf8(digitos).unwrap_or_else(|_| "0".to_string())
}

/// Decodes one `Day_ddmm=...` line into its date string and 24 raw integers.
pub(crate) fn decodificar_linea_dia(linea: &str) -> Result<(String, Vec<i64>)> {
    let trimmed = linea.trim();
    if !trimmed.starts_with("Day_") {
        return Err(Error::Format(format!(
            "Linea no comienza con 'Day_': '{trimmed}'"
        )));
    }
    let (etiqueta_raw, cuerpo) = trimmed
        .split_once('=')
        .ok_or_else(|| Error::Format(format!("Separador '=' no encontrado en: '{trimmed}'")))?;
    let dia = etiqueta_raw.strip_prefix("Day_").unwrap_or_default();
    if dia.len() != LONGITUD_ETIQUETA_DIA || !dia.chars().all(|c| c.is_ascii_digit()) {
        return Err(Error::Format(format!("Etiqueta de dia invalida: '{dia}'")));
    }
    let valores = decodificar_tokens_dia(cuerpo)?;
    if valores.len() != TOKENS_POR_DIA {
        return Err(Error::Format(format!(
            "Dia '{dia}' contiene {} valores; se esperaban {TOKENS_POR_DIA}",
            valores.len()
        )));
    }
    Ok((dia.to_string(), valores))
}

fn decodificar_tokens_dia(cuerpo: &str) -> Result<Vec<i64>> {
    if cuerpo.is_empty() {
        return Err(Error::Format("Cuerpo de dia vacio".into()));
    }
    let bytes = cuerpo.as_bytes();
    if bytes[0] != b'+' && bytes[0] != b'-' {
        return Err(Error::Format(format!(
            "Cuerpo de dia debe iniciar con '+' o '-': '{cuerpo}'"
        )));
    }
    let indices_signos: Vec<usize> = bytes
        .iter()
        .enumerate()
        .filter_map(|(i, &b)| {
            if b == b'+' || b == b'-' {
                Some(i)
            } else {
                None
            }
        })
        .collect();

    indices_signos
        .iter()
        .enumerate()
        .map(|(idx, &inicio)| {
            let fin = indices_signos.get(idx + 1).copied().unwrap_or(bytes.len());
            let signo = bytes[inicio];
            let token_str = &cuerpo[inicio + 1..fin];
            let mag = decodificar_b62(token_str)?;
            if signo == b'+' {
                Ok(mag)
            } else {
                Ok(-mag)
            }
        })
        .collect()
}

/// Encodes 24 raw integers into a standard `        Day_ddmm=...\\r\\n` line.
pub(crate) fn codificar_linea_dia(dia: &str, valores: &[i64]) -> String {
    let cuerpo: String = valores
        .iter()
        .map(|&v| {
            if v >= 0 {
                format!("+{}", codificar_b62(v as u64))
            } else {
                format!("-{}", codificar_b62((-v) as u64))
            }
        })
        .collect();
    format!("{PREFIJO_ESPACIOS_DIA}Day_{dia}={cuerpo}\r\n")
}
