//! Decoding for legacy Delphi ASCII + Real48 MEF weather format specifications.

use super::dominio::{DatosMefLegacy, DatosVariableMef, MefLegacy, VariableMef};
use crate::error::{Error, Result};
use crate::format::decodificar_texto_flexible;

pub(crate) fn parsear_mef_legacy(bytes: &[u8]) -> Result<MefLegacy> {
    let texto = decodificar_texto_flexible(bytes);
    let lineas: Vec<&str> = texto.lines().map(|s| s.trim_end_matches('\r')).collect();
    if lineas.len() < 2 {
        return Err(Error::Invalid("Cabecera MEF legacy insuficiente".into()));
    }
    if !lineas[0].starts_with("pvMeteoFormat;") && !bytes.starts_with(b"\xb3\x04") {
        return Err(Error::Invalid("Cabecera de archivo MEF no válida".into()));
    }
    let comentario = extraer_comentario_legacy(lineas[0])?;
    let (version, calcul) = extraer_version_calcul_legacy(lineas[1])?;
    let variables = parsear_variables_legacy(&lineas)?;
    MefLegacy::nuevo(DatosMefLegacy {
        comentario,
        version,
        calcul,
        variables,
    })
}

fn extraer_comentario_legacy(linea0: &str) -> Result<String> {
    let comentario = if linea0.starts_with("pvMeteoFormat;") {
        linea0.split(';').nth(1).unwrap_or("").trim().to_string()
    } else {
        let limpio = linea0.trim_start_matches(|c: char| {
            c == '\u{FFFD}' || (c as u32) <= 0x20 || (c as u32) == 0x04
        });
        limpio.split(';').next().unwrap_or("").trim().to_string()
    };
    if comentario.is_empty() {
        return Err(Error::Invalid(
            "Comentario de MEF legacy no puede ser vacío".into(),
        ));
    }
    Ok(comentario)
}

fn extraer_version_calcul_legacy(linea1: &str) -> Result<(String, String)> {
    if !linea1.starts_with("Version ") || !linea1.contains("Calcul=") {
        return Err(Error::Invalid(
            "Línea de versión MEF legacy no válida".into(),
        ));
    }
    let raw_version = linea1.split(';').next().unwrap_or("").trim();
    let version = raw_version
        .strip_prefix("Version ")
        .unwrap_or(raw_version)
        .trim()
        .to_string();
    let raw_calcul = linea1.split(';').nth(1).unwrap_or("").trim();
    let calcul = raw_calcul
        .strip_prefix("Calcul=")
        .unwrap_or(raw_calcul)
        .trim()
        .to_string();
    Ok((version, calcul))
}

fn parsear_variables_legacy(lineas: &[&str]) -> Result<Vec<VariableMef>> {
    let mut variables = Vec::new();
    for l in lineas {
        let partes: Vec<&str> = l.split(';').collect();
        if partes.len() >= 6
            && (partes[1] == "Irrad" || partes[1] == "Temper" || partes[1] == "WindVel")
        {
            let col = partes[2]
                .parse::<usize>()
                .map_err(|_| Error::Invalid(format!("Columna inválida: '{}'", partes[2])))?;
            if col == 0 {
                return Err(Error::Invalid(
                    "Columna en MEF legacy debe ser mayor que 0".into(),
                ));
            }
            let mult = partes[5]
                .parse::<f64>()
                .map_err(|_| Error::Invalid(format!("Multiplicador inválido: '{}'", partes[5])))?;
            let unidad = match partes[1] {
                "Temper" => "DegC",
                "WindVel" => "m_s",
                _ => "W_m2",
            };
            variables.push(VariableMef::nueva(DatosVariableMef {
                ident: partes[0].to_string(),
                tipo: partes[1].to_string(),
                columna: col,
                multiplicador: mult,
                unidad: unidad.into(),
            })?);
        }
    }
    Ok(variables)
}
