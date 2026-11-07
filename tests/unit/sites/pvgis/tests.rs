use super::normalizacion::parsear_tmy;
use super::*;
use std::fs;

/// A complete PVGIS TMY in UTC; February comes from a leap year, as PVGIS
/// often picks it, and `G(h)` carries the UTC hour to trace the shift.
fn tmy_completo() -> Vec<String> {
    let dias = [31, 28, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31];
    let mut filas = Vec::with_capacity(normalizacion::HORAS_ANIO);
    for (i, n) in dias.iter().enumerate() {
        let (mes, anio) = (i + 1, if mes_es_febrero(i) { 2012 } else { 2019 });
        for d in 1..=*n {
            for h in 0..24 {
                filas.push(format!(
                    r#"{{"time(UTC)":"{anio}{mes:02}{d:02}:{h:02}00","G(h)":{h},"Gd(h)":0,"T2m":20.0,"WS10m":1.0}}"#
                ));
            }
        }
    }
    filas
}

fn mes_es_febrero(indice: usize) -> bool {
    indice == 1
}

fn json_tmy(filas: &[String]) -> String {
    format!(r#"{{"outputs":{{"tmy_hourly":[{}]}}}}"#, filas.join(","))
}

#[test]
fn should_shift_full_tmy_to_legal_time_starting_on_january_first() {
    let horas = parsear_tmy(&json_tmy(&tmy_completo()), -5).expect("TMY válido");
    assert_eq!(horas.len(), normalizacion::HORAS_ANIO);
    // 05:00 UTC is local midnight; 00:00 UTC on 1 January wraps to 31 December 19:00.
    assert_eq!(
        (horas[0].mes, horas[0].dia, horas[0].hh, horas[0].g),
        (1, 1, 0, 5.0)
    );
    let ultima = &horas[horas.len() - 1];
    assert_eq!((ultima.mes, ultima.dia, ultima.hh), (12, 31, 23));
    assert!(!horas.iter().any(|h| h.mes == 2 && h.dia == 29));
}

#[test]
fn should_reject_incomplete_duplicated_or_out_of_range_tmy() {
    let completo = tmy_completo();
    let incompleto = &completo[1..];
    let mut duplicado = completo.clone();
    duplicado[1] = duplicado[0].clone();
    let mut fecha_mala = completo.clone();
    fecha_mala[0] = fecha_mala[0].replace("20190101:0000", "20190230:0000");
    let mut irradiancia_mala = completo.clone();
    irradiancia_mala[0] = irradiancia_mala[0].replace(r#""G(h)":0"#, r#""G(h)":-3"#);
    for (caso, filas) in [
        ("incompleto", incompleto),
        ("duplicado", &duplicado[..]),
        ("fecha", &fecha_mala[..]),
        ("irradiancia", &irradiancia_mala[..]),
    ] {
        assert!(
            matches!(
                parsear_tmy(&json_tmy(filas), 0),
                Err(crate::error::Error::Format(_))
            ),
            "{caso}"
        );
    }
}

// Replacing a read-only file fails on Windows, which is what CI runs.
#[cfg(windows)]
#[test]
fn should_restore_previous_met_when_csv_publication_fails() {
    let tmp = tempfile::tempdir().expect("ws");
    let meteo = tmp.path().join("Meteo");
    fs::create_dir_all(&meteo).expect("Meteo");
    let csv_bloqueado = meteo.join("Bloqueado_PVGIS.csv");
    fs::write(&csv_bloqueado, "csv_anterior").expect("CSV previo");
    let mut permisos = fs::metadata(&csv_bloqueado).expect("meta").permissions();
    permisos.set_readonly(true);
    fs::set_permissions(&csv_bloqueado, permisos).expect("solo lectura");
    fs::write(meteo.join("Bloqueado_PVGIS.MET"), "met_anterior").expect("MET previo");
    let cfg = ConfiguracionStaging {
        ws: tmp.path(),
        nombre: "Bloqueado",
        final_met: meteo.join("Bloqueado_PVGIS.MET"),
        final_csv: meteo.join("Bloqueado_PVGIS.csv"),
    };
    let staging = StagingMeteo::preparar(&cfg).expect("preparar staging");
    fs::write(&staging.csv_path, "csv_nuevo").expect("csv");
    fs::write(&staging.staging_met_path, "met_nuevo").expect("met");

    assert!(matches!(
        staging.commit(),
        Err(crate::error::Error::Invalid(_))
    ));
    assert_eq!(
        fs::read_to_string(&cfg.final_met).expect("MET"),
        "met_anterior"
    );
}
