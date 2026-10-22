//! Delphi Real48 6-byte floating point encoder and decoder.

use crate::error::{Error, Result};

/// Number of bytes per Delphi Real48 floating-point value.
pub(crate) const BYTES_POR_SLOT: usize = 6;

/// Exponent bias for Delphi Real48 (excess-129).
const EXPONENTE_SESGO_REAL48: i32 = 129;

/// Delphi Real48 mantissa size in bits.
const BITS_MANTISA_REAL48: i32 = 39;

/// Mask for the sign bit stored in the top bit of byte 5.
const MASCARA_SIGNO_REAL48: u8 = 0x80;

/// Mask for the mantissa bits stored in byte 5 (bits 0..6).
const MASCARA_MANTISA_BYTE5: u8 = 0x7F;

/// IEEE-754 binary64 mantissa bits.
const BITS_MANTISA_IEEE: u32 = 52;

/// IEEE-754 binary64 exponent mask (11 bits).
const MASCARA_EXP_IEEE: u64 = 0x7FF;

/// IEEE-754 binary64 mantissa mask (52 bits).
const MASCARA_MANTISA_IEEE: u64 = (1u64 << BITS_MANTISA_IEEE) - 1;

/// IEEE-754 binary64 exponent bias.
const SESGO_EXP_IEEE: i32 = 1023;

/// Maximum excess-129 exponent representable in Delphi Real48.
const MAX_EXPONENTE_SESGADO_REAL48: i32 = 255;

/// Decode one Delphi `Real48` (6-byte real): 1 exponent byte (excess-129),
/// 39-bit mantissa, top bit of byte 5 is the sign.
pub(crate) fn real48(b: &[u8; 6]) -> f64 {
    if b[0] == 0 {
        return 0.0;
    }
    let exp = i32::from(b[0]) - EXPONENTE_SESGO_REAL48;
    let mantissa = u64::from_le_bytes([
        b[1],
        b[2],
        b[3],
        b[4],
        b[5] & MASCARA_MANTISA_BYTE5,
        0,
        0,
        0,
    ]);
    let m = 1.0 + (mantissa as f64) / 2f64.powi(BITS_MANTISA_REAL48);
    let sign = if b[5] & MASCARA_SIGNO_REAL48 != 0 {
        -1.0
    } else {
        1.0
    };
    sign * m * 2f64.powi(exp)
}

/// Encode an `f64` into a 6-byte Delphi `Real48` representation.
pub(crate) fn encode_real48(v: f64) -> Result<[u8; 6]> {
    if v == 0.0 {
        return Ok([0u8; 6]);
    }
    if v.is_nan() {
        return Err(Error::Invalid("Real48 no soporta NaN".into()));
    }
    if v.is_infinite() {
        return Err(Error::Invalid(format!("Real48 no soporta infinito: {v}")));
    }

    let sign_bit = if v.is_sign_negative() {
        MASCARA_SIGNO_REAL48
    } else {
        0u8
    };
    let (biased, mantissa) = calcular_mantisa_y_sesgo(v.abs())?;
    Ok([
        biased,
        mantissa as u8,
        (mantissa >> 8) as u8,
        (mantissa >> 16) as u8,
        (mantissa >> 24) as u8,
        (((mantissa >> 32) as u8) & MASCARA_MANTISA_BYTE5) | sign_bit,
    ])
}

fn calcular_mantisa_y_sesgo(abs_v: f64) -> Result<(u8, u64)> {
    let bits = abs_v.to_bits();
    let exp_ieee = ((bits >> BITS_MANTISA_IEEE) & MASCARA_EXP_IEEE) as i32;
    let mant_ieee = bits & MASCARA_MANTISA_IEEE;

    if exp_ieee == 0 {
        return Err(Error::Invalid(format!(
            "Real48 underflow: valor {abs_v} es subnormal y no representable"
        )));
    }

    let mut exponent = exp_ieee - SESGO_EXP_IEEE;
    let m = 1.0 + (mant_ieee as f64) / 2f64.powi(BITS_MANTISA_IEEE as i32);
    let mantissa_f = (m - 1.0) * 2f64.powi(BITS_MANTISA_REAL48);
    let mut mantissa = mantissa_f.round() as u64;

    if mantissa == (1u64 << BITS_MANTISA_REAL48) {
        mantissa = 0;
        exponent += 1;
    }

    let biased = exponent + EXPONENTE_SESGO_REAL48;
    if biased > MAX_EXPONENTE_SESGADO_REAL48 {
        return Err(Error::Invalid(format!(
            "Real48 overflow: valor {abs_v} excede el máximo representable (~1.70e38)"
        )));
    }
    if biased < 1 {
        return Err(Error::Invalid(format!(
            "Real48 underflow: valor {abs_v} es inferior al mínimo representable (~2.94e-39)"
        )));
    }
    Ok((biased as u8, mantissa))
}
