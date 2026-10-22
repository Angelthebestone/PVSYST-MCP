//! Parsing for the PVsyst currency exchange rate table (`Currency.dat`).

use super::dominio::{DatosTablaDivisas, EntradaDivisa, TablaDivisas};
use crate::error::{Error, Result};
use crate::format::decodificar_latin1;

const DELIMITADOR_SEMICOLON: u8 = b';';
const CABECERA_EXACTA_V2: &str = "CURRENCY_RATES_V2";
const MIN_BYTES_POR_FILA: usize = 5;
const CAPACIDAD_INICIAL_SEGURA: usize = 64;

/// Parses raw bytes of `Currency.dat` into a structured [`TablaDivisas`].
pub(crate) fn parse_currency(bytes: &[u8]) -> Result<TablaDivisas> {
    if bytes.is_empty() {
        return Err(Error::Invalid("Archivo Currency.dat vacío".into()));
    }
    let mut lineas = bytes.split(|&b| b == b'\n');
    let linea_version = lineas
        .next()
        .map(limpiar_cr)
        .ok_or_else(|| Error::Invalid("Archivo Currency.dat vacío".into()))?;
    let version_str = String::from_utf8_lossy(linea_version)
        .trim_end_matches(';')
        .trim()
        .to_string();
    if version_str != CABECERA_EXACTA_V2 {
        return Err(Error::Invalid(format!(
            "Cabecera de divisas no reconocida: '{version_str}'"
        )));
    }

    let linea_control = lineas
        .next()
        .map(limpiar_cr)
        .ok_or_else(|| Error::Invalid("Falta línea de control en Currency.dat".into()))?;
    let (num_filas, base_idx, flag) = parsear_linea_control(linea_control, bytes.len())?;

    let mut divisas = Vec::with_capacity(num_filas.min(CAPACIDAD_INICIAL_SEGURA));
    for raw_line in lineas {
        let limpia = limpiar_cr(raw_line);
        if limpia.is_empty() {
            continue;
        }
        let entrada = parsear_linea_divisa(limpia)?;
        divisas.push(entrada);
    }

    if divisas.len() != num_filas {
        return Err(Error::Invalid(format!(
            "Discrepancia en conteo de divisas: esperado {num_filas}, leídas {}",
            divisas.len()
        )));
    }

    TablaDivisas::nueva(DatosTablaDivisas {
        version_formato: version_str,
        moneda_base_idx: base_idx,
        flag,
        divisas,
    })
}

fn limpiar_cr(b: &[u8]) -> &[u8] {
    if let Some(stripped) = b.strip_suffix(b"\r") {
        stripped
    } else {
        b
    }
}

fn parsear_linea_control(linea: &[u8], bytes_totales: usize) -> Result<(usize, usize, u32)> {
    let texto = String::from_utf8_lossy(linea);
    let partes: Vec<&str> = texto.split(';').map(str::trim).collect();
    if partes.len() < 3 {
        return Err(Error::Invalid(format!(
            "Línea de control inválida en Currency.dat: '{texto}'"
        )));
    }
    let num_filas = partes[0]
        .parse::<usize>()
        .map_err(|_| Error::Invalid(format!("Número de filas inválido: '{}'", partes[0])))?;

    let max_filas_posibles = bytes_totales / MIN_BYTES_POR_FILA;
    if num_filas > max_filas_posibles {
        return Err(Error::Invalid(format!(
            "Número de filas sospechoso en Currency.dat: {num_filas} filas para {bytes_totales} bytes"
        )));
    }

    let base_idx = partes[1]
        .parse::<usize>()
        .map_err(|_| Error::Invalid(format!("Índice base inválido: '{}'", partes[1])))?;
    let flag = partes[2]
        .parse::<u32>()
        .map_err(|_| Error::Invalid(format!("Flag de control inválido: '{}'", partes[2])))?;
    Ok((num_filas, base_idx, flag))
}

fn parsear_linea_divisa(linea: &[u8]) -> Result<EntradaDivisa> {
    let mut partes: Vec<&[u8]> = linea.split(|&b| b == DELIMITADOR_SEMICOLON).collect();
    if partes.last() == Some(&&b""[..]) {
        partes.pop();
    }
    if partes.len() != 3 {
        return Err(Error::Invalid(format!(
            "Línea de divisa debe contener exactamente 3 campos (código, nombre, tasa): '{}'",
            String::from_utf8_lossy(linea)
        )));
    }

    let codigo = String::from_utf8_lossy(partes[0]).trim().to_string();
    let nombre = decodificar_latin1(partes[1]);
    let tasa_str = String::from_utf8_lossy(partes[2]);
    let tasa_num = tasa_str
        .trim()
        .parse::<f64>()
        .map_err(|_| Error::Invalid(format!("Tasa de cambio no numérica: '{tasa_str}'")))?;

    EntradaDivisa::nueva(&codigo, nombre, tasa_num)
}
