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
    let png = png_rgb(2, 1, &rgb).expect("valid png");
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

#[test]
fn should_reject_zero_dimensions_without_panic() {
    let res = png_rgb(0, 0, &[]);
    assert!(res.is_err());
}
