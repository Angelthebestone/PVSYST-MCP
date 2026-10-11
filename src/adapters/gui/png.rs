//! PNG encoding and chunk assembly for GUI captures.

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
pub(crate) fn png_rgb(ancho: u32, alto: u32, rgb: &[u8]) -> Vec<u8> {
    let stride = ancho as usize * 3;
    let mut raw = Vec::with_capacity((stride + 1) * alto as usize);
    for fila in rgb.chunks(stride) {
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
    out
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
mod tests {
    use super::*;

    /// Returns (type, data) of every chunk in a PNG byte stream.
    fn chunks(png: &[u8]) -> Vec<([u8; 4], Vec<u8>)> {
        let mut out = Vec::new();
        let mut pos = 8;
        while pos + 12 <= png.len() {
            let len = u32::from_be_bytes(png[pos..pos + 4].try_into().unwrap()) as usize;
            let tipo: [u8; 4] = png[pos + 4..pos + 8].try_into().unwrap();
            out.push((tipo, png[pos + 8..pos + 8 + len].to_vec()));
            pos += 12 + len;
        }
        out
    }

    #[test]
    fn crc32_matches_known_iend_value() {
        assert_eq!(crc32(b"IEND"), 0xAE42_6082);
    }

    #[test]
    fn png_has_signature_header_and_decodable_pixels() {
        let rgb = [1u8, 2, 3, 4, 5, 6];
        let png = png_rgb(2, 1, &rgb);
        assert_eq!(&png[..8], &PNG_FIRMA);
        let parts = chunks(&png);
        assert_eq!(parts[0].0, *b"IHDR");
        assert_eq!(&parts[0].1[..8], &[0, 0, 0, 2, 0, 0, 0, 1]);
        assert_eq!(parts[0].1[8..], [8, 2, 0, 0, 0]);
        assert_eq!(parts.last().map(|c| c.0), Some(*b"IEND"));
        let idat = parts.iter().find(|c| &c.0 == b"IDAT").unwrap();
        let raw = miniz_oxide::inflate::decompress_to_vec_zlib(&idat.1).unwrap();
        assert_eq!(raw, [0, 1, 2, 3, 4, 5, 6]);
    }
}
