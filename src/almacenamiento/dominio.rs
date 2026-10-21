//! Battery-storage parameters and the `pvSystem` / `pvGridStorage` changes
//! that configure them.

use crate::args::{missing, opt_f64, opt_u32};
use crate::error::{Error, Result};
use crate::format::edit::EdicionObjeto;
use serde_json::{json, Value};

pub(crate) const BATERIA_DEFECTO: &str = "Generic_LI_Module_180Ah.BTR";
pub(crate) const N_SERIE_DEFECTO: u32 = 10;
pub(crate) const N_PARALELO_DEFECTO: u32 = 20;
pub(crate) const P_MAX_CARGA_DEFECTO_KW: f64 = 380.0;
pub(crate) const P_MAX_DESCARGA_DEFECTO_KW: f64 = 170.0;
pub(crate) const SOC_INICIAL_DEFECTO: f64 = 0.6;
const DECIMALES_SOC: usize = 3;
const DECIMALES_POTENCIA: usize = 3;

/// How the battery is dispatched.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Estrategia {
    Autoconsumo,
    RecortePicos,
}

impl Estrategia {
    pub(crate) fn de_texto(texto: &str) -> Result<Self> {
        match texto.trim().to_ascii_lowercase().as_str() {
            "autoconsumo" => Ok(Self::Autoconsumo),
            "recorte_picos" => Ok(Self::RecortePicos),
            _ => Err(Error::Invalid(format!(
                "Estrategia '{texto}' no válida; usa autoconsumo o recorte_picos"
            ))),
        }
    }

    fn nombre(self) -> &'static str {
        match self {
            Self::Autoconsumo => "autoconsumo",
            Self::RecortePicos => "recorte_picos",
        }
    }

    /// `Strategy`, `ChargingStrategy` and `DischStrategy` PVsyst values.
    fn campos(self) -> (&'static str, &'static str, &'static str) {
        match self {
            Self::Autoconsumo => ("SelfConsumption", "PVAvailable", "Immediate"),
            Self::RecortePicos => ("PeakShaving", "PVAboveThresh", "Immediate"),
        }
    }
}

/// Raw storage arguments.
#[derive(Debug, Clone)]
pub(crate) struct EntradaAlmacenamiento {
    pub(crate) estrategia: Option<String>,
    pub(crate) bateria: Option<String>,
    pub(crate) n_serie: Option<u32>,
    pub(crate) n_paralelo: Option<u32>,
    pub(crate) p_max_carga_kw: Option<f64>,
    pub(crate) p_max_descarga_kw: Option<f64>,
    pub(crate) soc_inicial: Option<f64>,
}

impl EntradaAlmacenamiento {
    pub(crate) fn de_args(args: &Value) -> Result<Self> {
        Ok(Self {
            estrategia: texto(args, "estrategia"),
            bateria: texto(args, "bateria"),
            n_serie: opt_u32(args, "n_serie")?,
            n_paralelo: opt_u32(args, "n_paralelo")?,
            p_max_carga_kw: opt_f64(args, "p_max_carga_kw")?,
            p_max_descarga_kw: opt_f64(args, "p_max_descarga_kw")?,
            soc_inicial: opt_f64(args, "soc_inicial")?,
        })
    }
}

fn texto(args: &Value, clave: &str) -> Option<String> {
    args.get(clave)
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .map(str::to_string)
}

fn validar_bateria(bateria: &str) -> Result<()> {
    if bateria.contains(['/', '\\', ':']) || !bateria.to_ascii_uppercase().ends_with(".BTR") {
        return Err(Error::Invalid(format!(
            "Nombre de batería inválido '{bateria}': debe terminar en .BTR y no contener separadores de ruta"
        )));
    }
    Ok(())
}

/// Validated battery-storage configuration.
#[derive(Debug, Clone, PartialEq)]
pub(crate) struct ParametrosAlmacenamiento {
    estrategia: Estrategia,
    bateria: String,
    n_serie: u32,
    n_paralelo: u32,
    p_max_carga_kw: f64,
    p_max_descarga_kw: f64,
    soc_inicial: f64,
}

impl ParametrosAlmacenamiento {
    pub(crate) fn nuevo(entrada: EntradaAlmacenamiento) -> Result<Self> {
        let estrategia = entrada
            .estrategia
            .as_deref()
            .ok_or_else(|| missing("estrategia", "autoconsumo o recorte_picos"))?;
        let estrategia = Estrategia::de_texto(estrategia)?;
        let bateria = entrada
            .bateria
            .unwrap_or_else(|| BATERIA_DEFECTO.to_string());
        validar_bateria(&bateria)?;
        let n_serie = entrada.n_serie.unwrap_or(N_SERIE_DEFECTO);
        let n_paralelo = entrada.n_paralelo.unwrap_or(N_PARALELO_DEFECTO);
        if n_serie == 0 || n_paralelo == 0 {
            return Err(Error::Invalid(
                "'n_serie' y 'n_paralelo' deben ser al menos 1".into(),
            ));
        }
        let p_max_carga_kw = entrada.p_max_carga_kw.unwrap_or(P_MAX_CARGA_DEFECTO_KW);
        let p_max_descarga_kw = entrada
            .p_max_descarga_kw
            .unwrap_or(P_MAX_DESCARGA_DEFECTO_KW);
        if p_max_carga_kw <= 0.0 || p_max_descarga_kw <= 0.0 {
            return Err(Error::Invalid(
                "Las potencias máxima de carga y descarga deben ser mayores que 0".into(),
            ));
        }
        let soc_inicial = entrada.soc_inicial.unwrap_or(SOC_INICIAL_DEFECTO);
        if !(soc_inicial > 0.0 && soc_inicial < 1.0) {
            return Err(Error::Invalid(format!(
                "'soc_inicial' debe estar entre 0 y 1 (exclusivo): {soc_inicial}"
            )));
        }
        Ok(Self {
            estrategia,
            bateria,
            n_serie,
            n_paralelo,
            p_max_carga_kw,
            p_max_descarga_kw,
            soc_inicial,
        })
    }

    pub(crate) fn bateria(&self) -> &str {
        &self.bateria
    }

    pub(crate) fn es_recorte_picos(&self) -> bool {
        self.estrategia == Estrategia::RecortePicos
    }

    pub(crate) fn como_json(&self) -> Value {
        json!({
            "estrategia": self.estrategia.nombre(),
            "bateria": self.bateria,
            "n_serie": self.n_serie,
            "n_paralelo": self.n_paralelo,
            "p_max_carga_kw": self.p_max_carga_kw,
            "p_max_descarga_kw": self.p_max_descarga_kw,
            "soc_inicial": self.soc_inicial,
        })
    }

    /// Object editions for `pvSystem` and `pvGridStorage`.
    pub(crate) fn ediciones(&self) -> Vec<EdicionObjeto> {
        let (strategy, charging, disch) = self.estrategia.campos();
        vec![
            EdicionObjeto {
                selector: "pvSystem".into(),
                cambios: vec![
                    ("BatteryFile".into(), Some(self.bateria.clone())),
                    ("NBattSerie".into(), Some(self.n_serie.to_string())),
                    ("NBattParall".into(), Some(self.n_paralelo.to_string())),
                ],
            },
            EdicionObjeto {
                selector: "pvGridStorage".into(),
                cambios: vec![
                    ("Strategy".into(), Some(strategy.to_string())),
                    ("ChargingStrategy".into(), Some(charging.to_string())),
                    ("DischStrategy".into(), Some(disch.to_string())),
                    (
                        "BattSoCInit".into(),
                        Some(format!("{:.*}", DECIMALES_SOC, self.soc_inicial)),
                    ),
                    (
                        "PMaxCharge".into(),
                        Some(format!("{:.*}", DECIMALES_POTENCIA, self.p_max_carga_kw)),
                    ),
                    (
                        "PMaxDisch".into(),
                        Some(format!("{:.*}", DECIMALES_POTENCIA, self.p_max_descarga_kw)),
                    ),
                ],
            },
        ]
    }
}

#[cfg(test)]
#[path = "../../tests/unit/almacenamiento/dominio.rs"]
mod tests;
