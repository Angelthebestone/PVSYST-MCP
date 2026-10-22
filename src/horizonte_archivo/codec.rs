//! Binary codec for serializing and deserializing PVsyst `.HOR` files.

use super::dominio::{validar_puntos, Horizonte, PuntoHorizonte, PUNTOS_MAXIMOS, PUNTOS_MINIMOS};
use crate::error::{Error, Result};
use crate::format::real48::{encode_real48, real48};
use crate::format::redondear;

/// Header magic bytes starting a Delphi serialized horizon file (0x04D7).
pub(crate) const HEADER_MAGIC: [u8; 2] = [0xd7, 0x04];

/// Profile magic bytes starting the TCubicProfile record (0x04A6).
pub(crate) const PROFILE_MAGIC: [u8; 2] = [0xa6, 0x04];

/// TCubicProfile profile mode byte (Mode=1).
pub(crate) const PROFILE_MODE: u8 = 0x01;

/// Number of bytes per Delphi Real48 floating-point value.
pub(crate) const BYTES_POR_REAL48: usize = 6;

/// Number of Real48 values per horizon point record.
pub(crate) const SLOTS_POR_PUNTO: usize = 5;

/// Bytes per horizon point record (5 * 6 = 30 bytes).
pub(crate) const BYTES_POR_REGISTRO: usize = BYTES_POR_REAL48 * SLOTS_POR_PUNTO;

/// Delphi constant factor for converting degrees to radians: 180 / PI rounded.
pub(crate) const FACTOR_GRADOS_A_RAD: f64 = 0.017_453_29;

/// Parses a complete `.HOR` file from bytes.
pub(crate) fn parse_horizonte(bytes: &[u8]) -> Result<Horizonte> {
    if bytes.len() < 7 {
        return Err(Error::Format("Archivo .HOR demasiado corto".into()));
    }
    if bytes[0..2] != HEADER_MAGIC {
        return Err(Error::Format(
            "Archivo .HOR inválido: firma de encabezado incorrecta".into(),
        ));
    }
    let crlf1 = find_crlf(bytes, 2)?;
    let crlf2 = find_crlf(bytes, crlf1 + 2)?;
    let titulo = parse_linea_titulo(&bytes[2..crlf1]);
    let (version, calcul) = parse_linea_version(&bytes[crlf1 + 2..crlf2])?;
    let puntos = parse_registros_puntos(&bytes[crlf2 + 2..])?;
    validar_puntos(&puntos)?;
    Ok(Horizonte {
        titulo,
        version,
        calcul,
        puntos,
    })
}

/// Encodes a [`Horizonte`] into the binary `.HOR` format.
pub(crate) fn encode_horizonte(h: &Horizonte) -> Result<Vec<u8>> {
    validar_puntos(&h.puntos)?;
    let count = h.puntos.len();
    if count > PUNTOS_MAXIMOS {
        return Err(Error::Invalid(format!(
            "Demasiados puntos para .HOR: {count} (máximo {PUNTOS_MAXIMOS})"
        )));
    }
    let line1 = format!("{};\r\n", h.titulo);
    let line2 = format!("Version {} ; Calcul={};\r\n", h.version, h.calcul);
    let bytes_line1 = codificar_latin1(&line1);
    let bytes_line2 = codificar_latin1(&line2);
    let total_tam =
        HEADER_MAGIC.len() + bytes_line1.len() + bytes_line2.len() + 5 + count * BYTES_POR_REGISTRO;

    let mut out = Vec::with_capacity(total_tam); // Preallocated buffer for binary serialization.
    out.extend_from_slice(&HEADER_MAGIC);
    out.extend_from_slice(&bytes_line1);
    out.extend_from_slice(&bytes_line2);
    out.extend_from_slice(&PROFILE_MAGIC);
    out.push(PROFILE_MODE);
    out.push(count as u8);
    out.push(count as u8);

    let pendientes = calcular_pendientes_perfil(&h.puntos)?;
    for (i, p) in h.puntos.iter().enumerate() {
        let az_rad = p.azimut_rad.unwrap_or(p.azimut * FACTOR_GRADOS_A_RAD);
        let ht_rad = p.altura_rad.unwrap_or(p.altura * FACTOR_GRADOS_A_RAD);
        let slope = pendientes[i];
        out.extend_from_slice(&encode_real48(az_rad)?);
        out.extend_from_slice(&encode_real48(ht_rad)?);
        out.extend_from_slice(&[0u8; 6]);
        out.extend_from_slice(&[0u8; 6]);
        out.extend_from_slice(&encode_real48(slope)?);
    }
    Ok(out)
}

fn find_crlf(bytes: &[u8], start: usize) -> Result<usize> {
    bytes
        .windows(2)
        .enumerate()
        .skip(start)
        .find(|(_, w)| w[0] == b'\r' && w[1] == b'\n')
        .map(|(i, _)| i)
        .ok_or_else(|| Error::Format("Encabezado .HOR incompleto: falta fin de línea CRLF".into()))
}

fn parse_linea_titulo(bytes: &[u8]) -> String {
    let raw = bytes.iter().map(|&b| b as char).collect::<String>();
    raw.strip_suffix(';').unwrap_or(&raw).trim().to_string()
}

fn parse_linea_version(bytes: &[u8]) -> Result<(String, String)> {
    let s = bytes.iter().map(|&b| b as char).collect::<String>();
    let parts: Vec<&str> = s
        .split(';')
        .map(str::trim)
        .filter(|p| !p.is_empty())
        .collect();
    let version = parts
        .first()
        .and_then(|p| p.strip_prefix("Version"))
        .map(str::trim)
        .ok_or_else(|| Error::Format("Campo 'Version' no encontrado en .HOR".into()))?
        .to_string();
    let calcul = parts
        .get(1)
        .and_then(|p| p.strip_prefix("Calcul="))
        .map(str::trim)
        .unwrap_or("$0001")
        .to_string();
    Ok((version, calcul))
}

fn parse_registros_puntos(bytes: &[u8]) -> Result<Vec<PuntoHorizonte>> {
    let prefix = bytes
        .get(..5)
        .ok_or_else(|| Error::Format("Encabezado de perfil .HOR truncado".into()))?;
    if prefix[0..2] != PROFILE_MAGIC || prefix[2] != PROFILE_MODE {
        return Err(Error::Format(
            "Firma de perfil TCubicProfile no coincide en .HOR".into(),
        ));
    }
    let count = prefix[4] as usize;
    let esperado = 5 + count * BYTES_POR_REGISTRO;
    if bytes.len() != esperado {
        return Err(Error::Format(format!(
            "Longitud inválida en .HOR: se esperaban {esperado} bytes de datos, hay {}",
            bytes.len()
        )));
    }
    (0..count)
        .map(|i| {
            let offset = 5 + i * BYTES_POR_REGISTRO;
            let chunk = &bytes[offset..offset + BYTES_POR_REGISTRO];
            decode_punto_registro(chunk)
        })
        .collect()
}

fn decode_punto_registro(chunk: &[u8]) -> Result<PuntoHorizonte> {
    let b_az: [u8; 6] = chunk[0..6]
        .try_into()
        .map_err(|_| Error::Format("Registro Real48 truncado".into()))?;
    let b_ht: [u8; 6] = chunk[6..12]
        .try_into()
        .map_err(|_| Error::Format("Registro Real48 truncado".into()))?;
    let b_sl: [u8; 6] = chunk[24..30]
        .try_into()
        .map_err(|_| Error::Format("Registro Real48 truncado".into()))?;
    let az_rad = real48(&b_az);
    let ht_rad = real48(&b_ht);
    let pendiente = real48(&b_sl);
    let az_deg = redondear(az_rad / FACTOR_GRADOS_A_RAD, 4);
    let ht_deg = redondear(ht_rad / FACTOR_GRADOS_A_RAD, 4);
    Ok(PuntoHorizonte {
        azimut: az_deg,
        altura: ht_deg,
        azimut_rad: Some(az_rad),
        altura_rad: Some(ht_rad),
        pendiente: Some(pendiente),
    })
}

fn calcular_pendientes_perfil(puntos: &[PuntoHorizonte]) -> Result<Vec<f64>> {
    let n = puntos.len();
    if n < PUNTOS_MINIMOS {
        return Err(Error::Invalid("Se requieren al menos 2 puntos".into()));
    }
    let mut slopes = Vec::with_capacity(n); // Preallocated accumulator for interval slopes.
    for i in 0..n - 1 {
        let p_curr = &puntos[i];
        let p_next = &puntos[i + 1];
        if let Some(s) = p_curr.pendiente {
            slopes.push(s);
            continue;
        }
        let x1 = p_curr
            .azimut_rad
            .unwrap_or(p_curr.azimut * FACTOR_GRADOS_A_RAD);
        let y1 = p_curr
            .altura_rad
            .unwrap_or(p_curr.altura * FACTOR_GRADOS_A_RAD);
        let x2 = p_next
            .azimut_rad
            .unwrap_or(p_next.azimut * FACTOR_GRADOS_A_RAD);
        let y2 = p_next
            .altura_rad
            .unwrap_or(p_next.altura * FACTOR_GRADOS_A_RAD);
        let dx = x2 - x1;
        if dx <= 0.0 {
            return Err(Error::Invalid(
                "Los puntos de azimut deben ser estrictamente crecientes".into(),
            ));
        }
        slopes.push((y2 - y1) / dx);
    }
    let ultimo_slope = puntos
        .last()
        .and_then(|p| p.pendiente)
        .unwrap_or_else(|| slopes.last().copied().unwrap_or(0.0));
    slopes.push(ultimo_slope);
    Ok(slopes)
}

fn codificar_latin1(s: &str) -> Vec<u8> {
    s.chars()
        .map(|c| if (c as u32) <= 0xFF { c as u8 } else { b'?' })
        .collect()
}
