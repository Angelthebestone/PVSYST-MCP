//! Domain models and invariants for PVsyst weather format specifications (`.MEF`).

use crate::error::{Error, Result};

/// Presence of UTF-8 Byte Order Mark in text representation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum PresenciaBom {
    Presente,
    Ausente,
}

/// DTO for creating a validated [`VariableMef`].
#[derive(Debug, Clone)]
pub(crate) struct DatosVariableMef {
    pub(crate) ident: String,
    pub(crate) tipo: String,
    pub(crate) columna: usize,
    pub(crate) multiplicador: f64,
    pub(crate) unidad: String,
}

/// Single variable column definition in a `.MEF` file.
#[derive(Debug, Clone, PartialEq)]
pub(crate) struct VariableMef {
    ident: String,
    tipo: String,
    columna: usize,
    multiplicador: f64,
    unidad: String,
}

impl VariableMef {
    /// Creates and validates a variable column mapping.
    pub(crate) fn nueva(datos: DatosVariableMef) -> Result<Self> {
        if datos.ident.trim().is_empty() {
            return Err(Error::Invalid(
                "Identificador de variable MEF no puede ser vacío".into(),
            ));
        }
        if datos.columna == 0 {
            return Err(Error::Invalid(
                "El índice de columna de variable MEF debe ser 1-based (> 0)".into(),
            ));
        }
        if !datos.multiplicador.is_finite() || datos.multiplicador <= 0.0 {
            return Err(Error::Invalid(format!(
                "Multiplicador de variable MEF debe ser finito y positivo: {}",
                datos.multiplicador
            )));
        }
        Ok(Self {
            ident: datos.ident,
            tipo: datos.tipo,
            columna: datos.columna,
            multiplicador: datos.multiplicador,
            unidad: datos.unidad,
        })
    }

    pub(crate) fn ident(&self) -> &str {
        &self.ident
    }

    pub(crate) fn tipo(&self) -> &str {
        &self.tipo
    }

    pub(crate) fn columna(&self) -> usize {
        self.columna
    }

    pub(crate) fn multiplicador(&self) -> f64 {
        self.multiplicador
    }

    pub(crate) fn unidad(&self) -> &str {
        &self.unidad
    }
}

/// Parameters for constructing a modern MEF specification.
#[derive(Debug, Clone)]
pub(crate) struct ParametrosMefModerno {
    pub(crate) comentario: String,
    pub(crate) version: String,
    pub(crate) formato_fecha: String,
    pub(crate) separador: String,
    pub(crate) paso_medicion_min: u32,
    pub(crate) tiempo_referencia: String,
    pub(crate) albedo: f64,
    pub(crate) columnas_fecha: Vec<usize>,
    pub(crate) variables: Vec<VariableMef>,
}

/// Modern `.MEF` file format (PVObject `pvMeteoFormat`).
#[derive(Debug, Clone, PartialEq)]
pub(crate) struct MefModerno {
    comentario: String,
    version: String,
    formato_fecha: String,
    separador: String,
    paso_medicion_min: u32,
    tiempo_referencia: String,
    albedo: f64,
    columnas_fecha: Vec<usize>,
    variables: Vec<VariableMef>,
    lineas_raw: Option<Vec<String>>,
    bom: PresenciaBom,
}

impl MefModerno {
    /// Creates and validates a modern MEF specification without raw lines.
    pub(crate) fn nuevo(params: ParametrosMefModerno) -> Result<Self> {
        Self::nuevo_con_crudos(params, None, PresenciaBom::Presente)
    }

    /// Creates a modern MEF specification preserving raw lines for exact round-trip.
    pub(crate) fn nuevo_con_crudos(
        params: ParametrosMefModerno,
        lineas_raw: Option<Vec<String>>,
        bom: PresenciaBom,
    ) -> Result<Self> {
        if params.comentario.trim().is_empty() {
            return Err(Error::Invalid(
                "Comentario de MEF no puede ser vacío".into(),
            ));
        }
        if params.paso_medicion_min == 0 {
            return Err(Error::Invalid(
                "Paso de medición de MEF debe ser mayor que 0".into(),
            ));
        }
        if params.variables.is_empty() {
            return Err(Error::Invalid(
                "MEF debe definir al menos una variable meteorológica".into(),
            ));
        }
        if !params.albedo.is_finite() || !(0.0..=1.0).contains(&params.albedo) {
            return Err(Error::Invalid(format!(
                "Albedo de MEF debe ser un número finito entre 0.0 y 1.0: {}",
                params.albedo
            )));
        }
        Ok(Self {
            comentario: params.comentario,
            version: params.version,
            formato_fecha: params.formato_fecha,
            separador: params.separador,
            paso_medicion_min: params.paso_medicion_min,
            tiempo_referencia: params.tiempo_referencia,
            albedo: params.albedo,
            columnas_fecha: params.columnas_fecha,
            variables: params.variables,
            lineas_raw,
            bom,
        })
    }

    pub(crate) fn comentario(&self) -> &str {
        &self.comentario
    }

    pub(crate) fn version(&self) -> &str {
        &self.version
    }

    pub(crate) fn formato_fecha(&self) -> &str {
        &self.formato_fecha
    }

    pub(crate) fn separador(&self) -> &str {
        &self.separador
    }

    pub(crate) fn paso_medicion_min(&self) -> u32 {
        self.paso_medicion_min
    }

    pub(crate) fn tiempo_referencia(&self) -> &str {
        &self.tiempo_referencia
    }

    pub(crate) fn albedo(&self) -> f64 {
        self.albedo
    }

    pub(crate) fn columnas_fecha(&self) -> &[usize] {
        &self.columnas_fecha
    }

    pub(crate) fn variables(&self) -> &[VariableMef] {
        &self.variables
    }

    pub(crate) fn lineas_raw(&self) -> Option<&[String]> {
        self.lineas_raw.as_deref()
    }

    pub(crate) fn bom(&self) -> PresenciaBom {
        self.bom
    }
}

/// DTO for creating a validated legacy MEF representation.
#[derive(Debug, Clone)]
pub(crate) struct DatosMefLegacy {
    pub(crate) comentario: String,
    pub(crate) version: String,
    pub(crate) calcul: String,
    pub(crate) variables: Vec<VariableMef>,
}

/// Legacy `.MEF` file format (< 6.40, Delphi ASCII + Real48).
#[derive(Debug, Clone, PartialEq)]
pub(crate) struct MefLegacy {
    comentario: String,
    version: String,
    calcul: String,
    variables: Vec<VariableMef>,
}

impl MefLegacy {
    /// Creates and validates a legacy MEF representation.
    pub(crate) fn nuevo(datos: DatosMefLegacy) -> Result<Self> {
        if datos.comentario.trim().is_empty() {
            return Err(Error::Invalid(
                "Comentario de MEF legacy no puede ser vacío".into(),
            ));
        }
        // In legacy MEF (e.g. binary format like SIG_Geneva.MEF), variables may be
        // in Delphi Real48 binary records rather than semicolon text lines.
        Ok(Self {
            comentario: datos.comentario,
            version: datos.version,
            calcul: datos.calcul,
            variables: datos.variables,
        })
    }

    pub(crate) fn comentario(&self) -> &str {
        &self.comentario
    }

    pub(crate) fn version(&self) -> &str {
        &self.version
    }

    pub(crate) fn calcul(&self) -> &str {
        &self.calcul
    }

    pub(crate) fn variables(&self) -> &[VariableMef] {
        &self.variables
    }
}

/// Top-level representation of any `.MEF` file (modern or legacy).
#[derive(Debug, Clone, PartialEq)]
pub(crate) enum MeteoMef {
    Moderno(MefModerno),
    Legacy(MefLegacy),
}

impl MeteoMef {
    pub(crate) fn comentario(&self) -> &str {
        match self {
            Self::Moderno(m) => m.comentario(),
            Self::Legacy(l) => l.comentario(),
        }
    }

    pub(crate) fn version(&self) -> &str {
        match self {
            Self::Moderno(m) => m.version(),
            Self::Legacy(l) => l.version(),
        }
    }

    pub(crate) fn variables(&self) -> &[VariableMef] {
        match self {
            Self::Moderno(m) => m.variables(),
            Self::Legacy(l) => l.variables(),
        }
    }

    pub(crate) fn calcul(&self) -> Option<&str> {
        match self {
            Self::Moderno(_) => None,
            Self::Legacy(l) => Some(l.calcul()),
        }
    }
}
