//! The series file rewritten around a new `VarMeteo` body: header and footer are kept,
//! in the file's own encoding.

use crate::error::{Error, Result};

/// The file's own encoding: UTF-8 stays UTF-8, an ANSI file is written back as Latin-1.
pub(super) fn codificar_como(texto: &str, es_utf8: bool) -> Result<Vec<u8>> {
    if es_utf8 {
        return Ok(texto.as_bytes().to_vec());
    }
    crate::format::codificar_latin1(texto)
        .ok_or_else(|| Error::Format("El archivo ANSI no admite el texto resultante".into()))
}

pub(super) fn ensamblar_archivo(original: &str, nuevo_cuerpo: &str) -> Result<String> {
    let pos_start = original
        .find("VarMeteo, list of=")
        .ok_or_else(|| Error::Format("No se encontro seccion 'VarMeteo' en original".into()))?;
    let line_start = original[..pos_start].rfind('\n').map_or(0, |p| p + 1);

    let end_marker = "End of the List VarMeteo";
    let pos_end = original
        .find(end_marker)
        .ok_or_else(|| Error::Format("No se encontro 'End of the List VarMeteo'".into()))?;
    let line_end = original[pos_end..]
        .find('\n')
        .map_or(original.len(), |p| pos_end + p + 1);

    let header = &original[..line_start];
    let footer = &original[line_end..];
    Ok(format!("{header}{nuevo_cuerpo}{footer}"))
}
