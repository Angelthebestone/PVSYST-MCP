//! Domain models and invariants for the PVsyst currency exchange rate table (`Currency.dat`).

use crate::error::{Error, Result};

const LONGITUD_CODIGO_DIVISA: usize = 3;

/// Single currency entry in the rates table.
#[derive(Debug, Clone, PartialEq)]
pub(crate) struct EntradaDivisa {
    codigo: String,
    nombre: String,
    tasa_cambio: f64,
}

impl EntradaDivisa {
    /// Creates a validated currency entry.
    pub(crate) fn nueva(codigo: &str, nombre: String, tasa_cambio: f64) -> Result<Self> {
        let cod = codigo.trim();
        if cod.len() != LONGITUD_CODIGO_DIVISA || !cod.chars().all(|c| c.is_ascii_alphabetic()) {
            return Err(Error::Invalid(format!(
                "Código de divisa debe tener exactamente {LONGITUD_CODIGO_DIVISA} letras ASCII: '{codigo}'"
            )));
        }
        if nombre.trim().is_empty() {
            return Err(Error::Invalid("Nombre de divisa no puede ser vacío".into()));
        }
        if !tasa_cambio.is_finite() || tasa_cambio <= 0.0 {
            return Err(Error::Invalid(format!(
                "Tasa de cambio debe ser un número finito positivo: {tasa_cambio}"
            )));
        }
        Ok(Self {
            codigo: cod.to_ascii_uppercase(),
            nombre,
            tasa_cambio,
        })
    }

    pub(crate) fn codigo(&self) -> &str {
        &self.codigo
    }

    pub(crate) fn nombre(&self) -> &str {
        &self.nombre
    }

    pub(crate) fn tasa_cambio(&self) -> f64 {
        self.tasa_cambio
    }
}

/// DTO for creating a validated [`TablaDivisas`].
#[derive(Debug, Clone)]
pub(crate) struct DatosTablaDivisas {
    pub(crate) version_formato: String,
    pub(crate) moneda_base_idx: usize,
    pub(crate) flag: u32,
    pub(crate) divisas: Vec<EntradaDivisa>,
}

/// Table of currency exchange rates stored in `Currency.dat`.
#[derive(Debug, Clone, PartialEq)]
pub(crate) struct TablaDivisas {
    version_formato: String,
    moneda_base_idx: usize,
    flag: u32,
    divisas: Vec<EntradaDivisa>,
}

impl TablaDivisas {
    /// Creates and validates a currency exchange table.
    pub(crate) fn nueva(datos: DatosTablaDivisas) -> Result<Self> {
        if datos.version_formato.trim().is_empty() {
            return Err(Error::Invalid(
                "La versión del formato de divisas no puede ser vacía".into(),
            ));
        }
        if datos.divisas.is_empty() {
            return Err(Error::Invalid(
                "La tabla de divisas debe contener al menos una entrada".into(),
            ));
        }
        if datos.moneda_base_idx == 0 || datos.moneda_base_idx > datos.divisas.len() {
            return Err(Error::Invalid(format!(
                "Índice de moneda base fuera de rango (1..={}): {}",
                datos.divisas.len(),
                datos.moneda_base_idx
            )));
        }
        Ok(Self {
            version_formato: datos.version_formato,
            moneda_base_idx: datos.moneda_base_idx,
            flag: datos.flag,
            divisas: datos.divisas,
        })
    }

    pub(crate) fn version_formato(&self) -> &str {
        &self.version_formato
    }

    pub(crate) fn moneda_base_idx(&self) -> usize {
        self.moneda_base_idx
    }

    pub(crate) fn flag(&self) -> u32 {
        self.flag
    }

    pub(crate) fn divisas(&self) -> &[EntradaDivisa] {
        &self.divisas
    }
}
