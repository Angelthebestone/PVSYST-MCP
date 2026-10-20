//! Monthly measured-vs-simulated comparison, independent of PVsyst files.

use crate::error::{Error, Result};
use crate::format::redondear;
use serde::Serialize;

pub(crate) const UMBRAL_DEFECTO_PCT: f64 = 10.0;
const MESES: usize = 12;
const DECIMALES_DESVIACION: i32 = 2;

/// Twelve monthly energy values in kWh.
#[derive(Debug, Clone, PartialEq)]
pub(crate) struct SerieMensual([f64; MESES]);

impl SerieMensual {
    /// Exactly 12 finite, non-negative values.
    pub(crate) fn nuevo(valores: &[f64]) -> Result<Self> {
        if valores.len() != MESES {
            return Err(Error::Invalid(format!(
                "La serie debe tener {MESES} valores; tiene {}",
                valores.len()
            )));
        }
        if let Some(valor) = valores
            .iter()
            .find(|valor| !valor.is_finite() || **valor < 0.0)
        {
            return Err(Error::Invalid(format!(
                "La serie solo admite valores finitos y no negativos: {valor}"
            )));
        }
        let array: [f64; MESES] = valores.try_into().map_err(|_| {
            Error::Invalid(format!("La serie debe tener exactamente {MESES} valores"))
        })?;
        Ok(Self(array))
    }

    fn suma(&self) -> f64 {
        self.0.iter().sum()
    }
}

/// One row of the comparison; `mes = 0` is the annual total.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub(crate) struct FilaMes {
    pub(crate) mes: u8,
    pub(crate) simulado_kwh: f64,
    pub(crate) medido_kwh: f64,
    pub(crate) desviacion_pct: Option<f64>,
}

/// Monthly rows plus the annual total and the months outside the threshold.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub(crate) struct ComparacionMensual {
    pub(crate) meses: Vec<FilaMes>,
    pub(crate) anual: FilaMes,
    pub(crate) fuera_de_umbral: Vec<u8>,
}

fn fila(mes: u8, simulado: f64, medido: f64) -> FilaMes {
    let desviacion_pct = (simulado != 0.0)
        .then(|| redondear((medido - simulado) / simulado * 100.0, DECIMALES_DESVIACION));
    FilaMes {
        mes,
        simulado_kwh: simulado,
        medido_kwh: medido,
        desviacion_pct,
    }
}

/// Compare both series; zero simulated months have no deviation.
pub(crate) fn comparar(
    simulada: &SerieMensual,
    medida: &SerieMensual,
    umbral_pct: f64,
) -> ComparacionMensual {
    let meses: Vec<FilaMes> = (0..MESES)
        .map(|i| {
            let mes = u8::try_from(i + 1).unwrap_or(u8::MAX);
            fila(mes, simulada.0[i], medida.0[i])
        })
        .collect();
    let anual = fila(0, simulada.suma(), medida.suma());
    let fuera_de_umbral = meses
        .iter()
        .filter(|fila| {
            fila.desviacion_pct
                .is_some_and(|desviacion| desviacion.abs() > umbral_pct)
        })
        .map(|fila| fila.mes)
        .collect();
    ComparacionMensual {
        meses,
        anual,
        fuera_de_umbral,
    }
}

fn segundo_valor(linea: &str) -> Option<f64> {
    let segundo = linea.split(';').nth(1)?.trim().replace(',', ".");
    segundo
        .parse::<f64>()
        .ok()
        .filter(|valor| valor.is_finite())
}

/// Parse `mes;energia_kwh` lines; an optional header is ignored.
pub(crate) fn serie_de_csv(texto: &str) -> Result<SerieMensual> {
    let lineas: Vec<&str> = texto
        .lines()
        .map(str::trim)
        .filter(|linea| !linea.is_empty())
        .collect();
    let inicio = usize::from(
        lineas
            .first()
            .is_some_and(|linea| segundo_valor(linea).is_none()),
    );
    let valores: Result<Vec<f64>> = lineas[inicio..]
        .iter()
        .map(|linea| {
            segundo_valor(linea)
                .ok_or_else(|| Error::Format(format!("Fila mensual no numérica: '{linea}'")))
        })
        .collect();
    SerieMensual::nuevo(&valores?)
}

#[cfg(test)]
#[path = "../../tests/unit/medicion/dominio.rs"]
mod tests;
