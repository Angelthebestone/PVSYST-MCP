//! PNG encoding and chunk assembly for GUI captures.

use crate::error::{Error, Result};

const PNG_FIRMA: [u8; 8] = [0x89, b'P', b'N', b'G', b'\r', b'\n', 0x1A, b'\n'];
const BIT_DEPTH_8: u8 = 8;
const COLOR_TYPE_RGB: u8 = 2;
const COMPRESSION_DEFLATE: u8 = 0;
const FILTER_NONE: u8 = 0;
const INTERLACE_NONE: u8 = 0;
const CRC_INIT: u32 = 0xFFFF_FFFF;
const CRC_POLY: u32 = 0xEDB8_8320;
const ZLIB_LEVEL_DEFAULT: u8 = 6;

/// Encodes packed 8-bit RGB pixels as a PNG (filter 0 on every row).
pub(crate) fn png_rgb(ancho: u32, alto: u32, rgb: &[u8]) -> Result<Vec<u8>> {
    if ancho == 0 || alto == 0 {
        return Err(Error::Gui(
            "Dimensiones de imagen inválidas para PNG: ancho y alto deben ser mayores que cero"
                .into(),
        ));
    }
    let stride = ancho as usize * 3;
    let esperado = stride.saturating_mul(alto as usize);
    if rgb.len() < esperado {
        return Err(Error::Gui(format!(
            "Búfer RGB insuficiente: esperado {esperado} bytes, recibido {}",
            rgb.len()
        )));
    }
    let mut raw = Vec::with_capacity((stride + 1) * alto as usize);
    for fila in rgb.chunks(stride).take(alto as usize) {
        raw.push(FILTER_NONE);
        raw.extend_from_slice(fila);
    }
    let idat = miniz_oxide::deflate::compress_to_vec_zlib(&raw, ZLIB_LEVEL_DEFAULT);
    let mut ihdr = Vec::with_capacity(13);
    ihdr.extend_from_slice(&ancho.to_be_bytes());
    ihdr.extend_from_slice(&alto.to_be_bytes());
    ihdr.extend_from_slice(&[
        BIT_DEPTH_8,
        COLOR_TYPE_RGB,
        COMPRESSION_DEFLATE,
        FILTER_NONE,
        INTERLACE_NONE,
    ]);
    let mut out = PNG_FIRMA.to_vec();
    push_chunk(&mut out, b"IHDR", &ihdr);
    push_chunk(&mut out, b"IDAT", &idat);
    push_chunk(&mut out, b"IEND", &[]);
    Ok(out)
}

/// Appends one PNG chunk: length, type, data and the CRC over type and data.
fn push_chunk(out: &mut Vec<u8>, tipo: &[u8; 4], datos: &[u8]) {
    out.extend_from_slice(&(datos.len() as u32).to_be_bytes());
    let mut cuerpo = tipo.to_vec();
    cuerpo.extend_from_slice(datos);
    out.extend_from_slice(&cuerpo);
    out.extend_from_slice(&crc32(&cuerpo).to_be_bytes());
}

/// CRC-32 (ISO 3309, as used by PNG).
pub(crate) fn crc32(datos: &[u8]) -> u32 {
    let mut crc = CRC_INIT;
    for &b in datos {
        crc ^= u32::from(b);
        for _ in 0..8 {
            let mascara = (crc & 1).wrapping_neg();
            crc = (crc >> 1) ^ (CRC_POLY & mascara);
        }
    }
    !crc
}

#[cfg(test)]
#[path = "../../../tests/unit/adapters/gui/png.rs"]
mod tests;
