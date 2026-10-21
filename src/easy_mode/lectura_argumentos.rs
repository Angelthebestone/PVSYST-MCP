//! Typed readers for the untyped JSON arguments of the easy-mode pipeline.

use crate::args::missing;
use crate::error::{Error, Result};
use serde_json::Value;

pub(crate) fn parsear_f64_requerido(args: &Value, claves: &[&str], pista: &str) -> Result<f64> {
    parsear_f64_opcional_sin_defecto(args, claves)?.ok_or_else(|| missing(claves[0], pista))
}

pub(crate) fn parsear_f64_opcional(args: &Value, claves: &[&str], defecto: f64) -> Result<f64> {
    Ok(parsear_f64_opcional_sin_defecto(args, claves)?.unwrap_or(defecto))
}

pub(crate) fn parsear_f64_opcional_sin_defecto(
    args: &Value,
    claves: &[&str],
) -> Result<Option<f64>> {
    for clave in claves {
        if let Some(valor) = args.get(*clave) {
            if valor.is_null() {
                continue;
            }
            if let Some(n) = valor.as_f64() {
                if !n.is_finite() {
                    return Err(Error::Invalid(format!(
                        "Valor no finito para '{clave}': {n}"
                    )));
                }
                return Ok(Some(n));
            }
            if let Some(s) = valor.as_str() {
                let recortado = s.trim();
                if recortado.is_empty() {
                    continue;
                }
                let parsed = recortado.parse::<f64>().map_err(|_| {
                    Error::Invalid(format!(
                        "Valor no numérico para '{clave}': se esperaba un número, se obtuvo '{s}'"
                    ))
                })?;
                if !parsed.is_finite() {
                    return Err(Error::Invalid(format!(
                        "Valor no finito para '{clave}': {parsed}"
                    )));
                }
                return Ok(Some(parsed));
            }
            return Err(Error::Invalid(format!(
                "Tipo no válido para '{clave}': se esperaba un número"
            )));
        }
    }
    Ok(None)
}

pub(crate) fn parsear_u32_opcional(args: &Value, claves: &[&str]) -> Result<Option<u32>> {
    for clave in claves {
        if let Some(valor) = args.get(*clave) {
            if valor.is_null() {
                continue;
            }
            if let Some(n) = valor.as_u64() {
                let u = u32::try_from(n).map_err(|_| {
                    Error::Invalid(format!(
                        "Valor entero fuera de rango para '{clave}': '{n}' excede el límite u32 ({})",
                        u32::MAX
                    ))
                })?;
                return Ok(Some(u));
            }
            if let Some(i) = valor.as_i64() {
                // `as_u64` failed, so the integer is negative.
                return Err(Error::Invalid(format!(
                    "Valor entero para '{clave}' no puede ser negativo: {i}"
                )));
            }
            if let Some(s) = valor.as_str() {
                let recortado = s.trim();
                if recortado.is_empty() {
                    continue;
                }
                let parsed = recortado.parse::<u32>().map_err(|_| {
                    Error::Invalid(format!("Valor entero no válido para '{clave}': '{s}'"))
                })?;
                return Ok(Some(parsed));
            }
            return Err(Error::Invalid(format!(
                "Tipo no válido para '{clave}': se esperaba un entero positivo"
            )));
        }
    }
    Ok(None)
}

pub(crate) fn parsear_bool_opcional(args: &Value, claves: &[&str], defecto: bool) -> Result<bool> {
    for clave in claves {
        if let Some(valor) = args.get(*clave) {
            if valor.is_null() {
                continue;
            }
            if let Some(b) = valor.as_bool() {
                return Ok(b);
            }
            if let Some(s) = valor.as_str() {
                match s.trim().to_lowercase().as_str() {
                    "true" | "1" | "si" | "yes" => return Ok(true),
                    "false" | "0" | "no" => return Ok(false),
                    _ => {
                        return Err(Error::Invalid(format!(
                            "Valor booleano no válido para '{clave}': '{s}'"
                        )))
                    }
                }
            }
            return Err(Error::Invalid(format!(
                "Tipo no válido para '{clave}': se esperaba un booleano"
            )));
        }
    }
    Ok(defecto)
}
