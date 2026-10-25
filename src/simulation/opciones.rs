//! Boundary checks for `simular` options, applied before any PVsystCLI line is built.
//! Values come from the documented PVsystCLI reference; anything else is rejected
//! with a message that names the parameter.

use crate::args::{no_encontrado, opt_str};
use crate::error::{Error, Result};
use serde_json::Value;
use std::path::Path;

const TIME_STEPS: [&str; 2] = ["hour", "subhour"];
const PAGINAS_REPORTE: [&str; 15] = [
    "cover",
    "summary",
    "notes",
    "params",
    "horizon",
    "shadings",
    "usersneeds",
    "results",
    "econeval",
    "circuit",
    "losses",
    "financialbalance",
    "p50p90",
    "carbonbalance",
    "predefgraphs",
];
const LOG_LEVEL_MIN: i64 = 0;
const LOG_LEVEL_MAX: i64 = 3;
const ANIO_ABREVIADO_BASE: u32 = 2000;
const ANIO_REFERENCIA_SIN_ANIO: u32 = 2000;

/// Rejects a variant that is not on disk, naming the variants that are.
pub(crate) fn exigir_variante(variant_file: &Path) -> Result<()> {
    if variant_file.exists() {
        return Ok(());
    }
    let stem = variant_file
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or_default();
    let disponibles = crate::projects::variantes_de(variant_file, stem);
    let pedido = variant_file
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or_default();
    let carpeta = variant_file
        .parent()
        .map(|p| p.display().to_string())
        .unwrap_or_default();
    Err(no_encontrado("Variante", pedido, &disponibles, &carpeta))
}

/// Validates every option of a simulation request that PVsystCLI would otherwise
/// silently clamp, skip or reject only after a 3-second run.
pub(crate) fn validar(args: &Value) -> Result<()> {
    validar_time_step(args)?;
    validar_log_level(args)?;
    validar_paginas(args)?;
    validar_fechas(args)
}

fn validar_time_step(args: &Value) -> Result<()> {
    match opt_str(args, &["time_step"]) {
        Some(v) if !TIME_STEPS.contains(&v) => Err(Error::Invalid(format!(
            "'time_step' debe ser 'hour' o 'subhour' (recibido '{v}')"
        ))),
        _ => Ok(()),
    }
}

pub(crate) fn validar_log_level(args: &Value) -> Result<()> {
    let Some(v) = args.get("log_level").and_then(Value::as_i64) else {
        return Ok(());
    };
    if (LOG_LEVEL_MIN..=LOG_LEVEL_MAX).contains(&v) {
        return Ok(());
    }
    Err(Error::Invalid(format!(
        "'log_level' debe estar entre {LOG_LEVEL_MIN} y {LOG_LEVEL_MAX} (recibido {v})"
    )))
}

fn validar_paginas(args: &Value) -> Result<()> {
    let Some(pags) = args.get("paginas_reporte").and_then(Value::as_array) else {
        return Ok(());
    };
    let invalidas: Vec<&str> = pags
        .iter()
        .filter_map(Value::as_str)
        .filter(|p| !PAGINAS_REPORTE.contains(p))
        .collect();
    if invalidas.is_empty() {
        return Ok(());
    }
    Err(Error::Invalid(format!(
        "'paginas_reporte' no admite {} (válidas: {})",
        invalidas.join(", "),
        PAGINAS_REPORTE.join(", ")
    )))
}

#[derive(Clone, Copy)]
struct Fecha {
    anio: Option<u32>,
    mes: u32,
    dia: u32,
}

impl Fecha {
    /// Total order only when the year is known (`MM-DD` has none).
    fn clave(self) -> Option<(u32, u32, u32)> {
        self.anio.map(|a| (a, self.mes, self.dia))
    }
}

fn validar_fechas(args: &Value) -> Result<()> {
    let inicio = fecha_opcional(args, "start_date")?;
    let fin = fecha_opcional(args, "end_date")?;
    let orden = inicio
        .zip(fin)
        .and_then(|(i, f)| Some((i.clave()?, f.clave()?)));
    match orden {
        Some((i, f)) if f < i => Err(Error::Invalid(
            "'end_date' es anterior a 'start_date'".to_string(),
        )),
        _ => Ok(()),
    }
}

fn fecha_opcional(args: &Value, clave: &str) -> Result<Option<Fecha>> {
    let Some(texto) = opt_str(args, &[clave]) else {
        return Ok(None);
    };
    parsear_fecha(texto).map(Some).ok_or_else(|| {
        Error::Invalid(format!(
            "'{clave}' no es una fecha válida: '{texto}' (formatos: AAAA-MM-DD, AA-MM-DD, MM-DD)"
        ))
    })
}

fn parsear_fecha(texto: &str) -> Option<Fecha> {
    let partes: Vec<u32> = texto
        .split(['-', '.'])
        .map(|p| p.parse().ok())
        .collect::<Option<Vec<u32>>>()?;
    let (anio, mes, dia) = match partes.as_slice() {
        [anio, mes, dia] => (Some(anio_completo(*anio)), *mes, *dia),
        [mes, dia] => (None, *mes, *dia),
        _ => return None,
    };
    dia_valido(anio, mes, dia).then_some(Fecha { anio, mes, dia })
}

fn anio_completo(anio: u32) -> u32 {
    if anio < 100 {
        ANIO_ABREVIADO_BASE + anio
    } else {
        anio
    }
}

fn dia_valido(anio: Option<u32>, mes: u32, dia: u32) -> bool {
    let referencia = anio.unwrap_or(ANIO_REFERENCIA_SIN_ANIO);
    let maximo = match mes {
        2 if es_bisiesto(referencia) => 29,
        2 => 28,
        4 | 6 | 9 | 11 => 30,
        1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
        _ => return false,
    };
    (1..=maximo).contains(&dia)
}

fn es_bisiesto(anio: u32) -> bool {
    anio % 4 == 0 && (anio % 100 != 0 || anio % 400 == 0)
}
