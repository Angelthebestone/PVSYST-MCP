//! Raw line helpers shared by the byte-preserving editors (`insert`, `delete`).

/// Lines of `bytes` split on `\n`; a `\r` stays at the end of its line.
pub(super) fn split(bytes: &[u8]) -> Vec<Vec<u8>> {
    bytes.split(|&b| b == b'\n').map(<[u8]>::to_vec).collect()
}

/// The inverse of [`split`].
pub(super) fn join(lines: &[Vec<u8>]) -> Vec<u8> {
    lines.join(&b'\n')
}

/// A line as trimmed text, for matching headers and keys.
pub(super) fn texto(line: &[u8]) -> String {
    String::from_utf8_lossy(line).trim().to_string()
}
