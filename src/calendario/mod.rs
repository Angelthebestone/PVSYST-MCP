//! Proleptic Gregorian calendar arithmetic shared by every feature that needs
//! dates, so there is exactly one implementation (Howard Hinnant's algorithms).

/// A civil calendar date (`year`, `month`, `day`).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct FechaCivil {
    anio: i64,
    mes: u32,
    dia: u32,
}

impl FechaCivil {
    /// Builds a date from already validated calendar fields.
    pub(crate) fn nuevo(anio: i64, mes: u32, dia: u32) -> FechaCivil {
        FechaCivil { anio, mes, dia }
    }

    pub(crate) fn anio(&self) -> i64 {
        self.anio
    }

    pub(crate) fn mes(&self) -> u32 {
        self.mes
    }

    pub(crate) fn dia(&self) -> u32 {
        self.dia
    }
}

/// Whether `anio` is a leap year in the proleptic Gregorian calendar.
pub(crate) fn es_bisiesto(anio: i64) -> bool {
    anio % 4 == 0 && (anio % 100 != 0 || anio % 400 == 0)
}

/// Days in `mes` of `anio`; 0 for a month outside 1..=12.
pub(crate) fn dias_del_mes(anio: i64, mes: u32) -> u32 {
    match mes {
        1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
        4 | 6 | 9 | 11 => 30,
        2 if es_bisiesto(anio) => 29,
        2 => 28,
        _ => 0,
    }
}

/// Civil date of a count of days since 1970-01-01 (Howard Hinnant's algorithm).
pub(crate) fn civil_desde_dias(dias: i64) -> FechaCivil {
    let z = dias + 719_468;
    let era = z.div_euclid(146_097);
    let doe = z.rem_euclid(146_097);
    let yoe = (doe - doe / 1_460 + doe / 36_524 - doe / 146_096) / 365;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let dia = (doy - (153 * mp + 2) / 5 + 1) as u32;
    let mes = if mp < 10 { mp + 3 } else { mp - 9 } as u32;
    let anio = yoe + era * 400 + i64::from(mes <= 2);
    FechaCivil { anio, mes, dia }
}

/// Days since 1970-01-01 of a civil date (Howard Hinnant's algorithm).
pub(crate) fn dias_desde_civil(fecha: FechaCivil) -> i64 {
    let anio = fecha.anio - i64::from(fecha.mes <= 2);
    let era = anio.div_euclid(400);
    let yoe = anio - era * 400;
    let mp = i64::from(fecha.mes) + if fecha.mes > 2 { -3 } else { 9 };
    let doy = (153 * mp + 2) / 5 + i64::from(fecha.dia) - 1;
    let doe = yoe * 365 + yoe / 4 - yoe / 100 + doy;
    era * 146_097 + doe - 719_468
}

#[cfg(test)]
#[path = "../../tests/unit/calendario/mod.rs"]
mod tests;
