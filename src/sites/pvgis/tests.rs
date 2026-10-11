use super::normalizacion::*;
use super::*;
use crate::format::edit;
use std::fs;
use std::path::Path;

#[test]
fn should_roundtrip_civil_dates() {
    assert_eq!(normalizacion::dias_desde_civil(1970, 1, 1), 0);
    for z in [-800_000, -1, 0, 1, 20_089, 40_000] {
        let (y, m, d) = normalizacion::civil_desde_dias(z);
        assert_eq!(normalizacion::dias_desde_civil(y, m, d), z);
    }
    assert_eq!(
        normalizacion::civil_desde_dias(normalizacion::dias_desde_civil(2009, 12, 31)),
        (2009, 12, 31)
    );
}

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
        assert!(parsear_tmy(&json_tmy(filas), 0).is_err(), "{caso}");
    }
}

#[test]
fn should_keep_limit_and_endpoints_when_decimating() {
    let p: Vec<(f64, f64)> = (0..250).map(|i| (f64::from(i), 1.0)).collect();
    let d = diezmar(p);
    assert_eq!(d.len(), normalizacion::MAX_PUNTOS);
    assert_eq!(d[0].0, 0.0);
    assert_eq!(d[normalizacion::MAX_PUNTOS - 1].0, 249.0);
}

#[test]
fn should_zero_surplus_existing_points_in_horizon_profile() {
    let c = normalizacion::cambios_perfil(&[(0.0, 1.0), (10.0, 2.0)], &[1, 2, 3, 4]);
    assert!(c.contains(&("Point_3".to_string(), Some("0.0,0.0".to_string()))));
    assert!(c.contains(&("Point_4".to_string(), Some("0.0,0.0".to_string()))));
    assert!(!c.iter().any(|(k, _)| k == "Point_5"));
    assert!(c.iter().all(|(_, v)| v.is_some()));
    assert!(c.contains(&("NPtsEff".to_string(), Some("2".to_string()))));
}

#[test]
fn should_reject_malformed_pvgis_timestamps() {
    for malo in [
        "2019010100000",
        "20190101-0000",
        "20191301:0000",
        "20190101:2400",
        "2019010a:0000",
    ] {
        assert!(normalizacion::fecha_utc(malo).is_none(), "{malo}");
    }
    assert_eq!(
        normalizacion::fecha_utc("20240229:2300"),
        Some((2024, 2, 29, 23, 0))
    );
}

fn contar_archivos(dir: &Path) -> usize {
    let mut count = 0;
    if let Ok(entries) = fs::read_dir(dir) {
        for entry in entries.flatten() {
            let p = entry.path();
            if p.is_file() {
                count += 1;
            } else if p.is_dir() {
                count += contar_archivos(&p);
            }
        }
    }
    count
}

#[test]
fn should_leave_no_files_in_workspace_on_conversion_failure() {
    let tmp_root = std::env::temp_dir().join(format!("test_pvgis_fail_{}", std::process::id()));
    let _ = fs::remove_dir_all(&tmp_root);
    fs::create_dir_all(&tmp_root).expect("crear tmp workspace");

    let cfg = ConfiguracionStaging {
        ws: &tmp_root,
        nombre: "SitioFallo",
        final_met: tmp_root.join("Meteo").join("SitioFallo_PVGIS.MET"),
        final_csv: tmp_root.join("Meteo").join("SitioFallo_PVGIS.csv"),
    };
    let staging = StagingMeteo::preparar(&cfg).expect("preparar staging");
    assert!(
        staging.staging_dir.exists(),
        "directorio staging debe existir"
    );

    // Write dummy staging artifacts
    fs::write(&staging.csv_path, "col1;col2\n10;20").expect("escribir csv staging");
    assert!(staging.csv_path.exists());

    // Trigger rollback to simulate failure path
    staging.rollback();

    // Verify: staging dir is gone and neither final MET nor final CSV were created
    assert!(!staging.staging_dir.exists(), "staging dir debe eliminarse");
    assert!(!cfg.final_met.exists(), "final MET no debe crearse");
    assert!(!cfg.final_csv.exists(), "final CSV no debe crearse");
    assert_eq!(
        contar_archivos(&tmp_root),
        0,
        "el workspace no debe contener ningún archivo tras el rollback"
    );

    let _ = fs::remove_dir_all(&tmp_root);
}

#[test]
fn should_commit_staging_files_atomically_on_success() {
    let tmp_root = std::env::temp_dir().join(format!("test_pvgis_succ_{}", std::process::id()));
    let _ = fs::remove_dir_all(&tmp_root);
    fs::create_dir_all(&tmp_root).expect("crear tmp workspace");

    let cfg = ConfiguracionStaging {
        ws: &tmp_root,
        nombre: "SitioExito",
        final_met: tmp_root.join("Meteo").join("SitioExito_PVGIS.MET"),
        final_csv: tmp_root.join("Meteo").join("SitioExito_PVGIS.csv"),
    };
    let staging = StagingMeteo::preparar(&cfg).expect("preparar staging");
    fs::write(&staging.csv_path, "csv_content").expect("write staging csv");
    fs::write(&staging.staging_met_path, "met_content").expect("write staging met");

    staging.commit().expect("commit exitoso");

    assert!(
        !staging.staging_dir.exists(),
        "staging dir debe eliminarse tras commit"
    );
    assert!(cfg.final_met.exists(), "final MET debe existir");
    assert!(cfg.final_csv.exists(), "final CSV debe existir");
    assert_eq!(fs::read_to_string(&cfg.final_met).unwrap(), "met_content");

    let _ = fs::remove_dir_all(&tmp_root);
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

    assert!(staging.commit().is_err());
    assert_eq!(
        fs::read_to_string(&cfg.final_met).expect("MET"),
        "met_anterior"
    );
}

#[test]
fn should_isolate_mef_filename_per_run() {
    let tmp_root = std::env::temp_dir().join(format!("test_pvgis_iso_{}", std::process::id()));
    let _ = fs::remove_dir_all(&tmp_root);
    fs::create_dir_all(&tmp_root).expect("crear tmp workspace");

    let cfg1 = ConfiguracionStaging {
        ws: &tmp_root,
        nombre: "S1",
        final_met: tmp_root.join("Meteo").join("S1.MET"),
        final_csv: tmp_root.join("Meteo").join("S1.csv"),
    };
    let cfg2 = ConfiguracionStaging {
        ws: &tmp_root,
        nombre: "S2",
        final_met: tmp_root.join("Meteo").join("S2.MET"),
        final_csv: tmp_root.join("Meteo").join("S2.csv"),
    };
    let s1 = StagingMeteo::preparar(&cfg1).unwrap();
    let s2 = StagingMeteo::preparar(&cfg2).unwrap();

    assert_ne!(s1.mef_path, s2.mef_path, "nombres de MEF deben ser únicos");
    assert_ne!(
        s1.staging_dir, s2.staging_dir,
        "directorios staging deben ser únicos"
    );

    s1.rollback();
    s2.rollback();
    let _ = fs::remove_dir_all(&tmp_root);
}

#[test]
fn should_not_touch_unrelated_bytes_in_horizon_profile_edit() {
    let ruta = crate::config::data_ro()
        .join("PVsyst8.1_Data")
        .join("Projects")
        .join("_DEMO_Residential.VC1");
    let Ok(bytes) = fs::read(&ruta) else {
        return;
    };
    let existentes = normalizacion::puntos_existentes(&bytes);
    assert!(!existentes.is_empty(), "la demo trae un horizonte");
    let puntos = [(-180.0, 1.0), (0.0, 2.0), (180.0, 1.0)];
    let (nuevo, _) = edit::edit_fields(
        &bytes,
        "pvHorizon",
        &normalizacion::cambios_perfil(&puntos, &existentes),
    )
    .expect("edición válida");
    let spans = edit::spans_of(&bytes);
    let h = &spans[edit::find_span(&spans, "pvHorizon").expect("pvHorizon")];
    let antes: Vec<&[u8]> = bytes.split(|b| *b == b'\n').collect();
    let despues: Vec<&[u8]> = nuevo.split(|b| *b == b'\n').collect();
    let n_menos = antes.len() - despues.len();
    assert!(antes[..h.open] == despues[..h.open], "antes del horizonte");
    assert!(
        antes[h.close + 1..] == despues[h.close + 1 - n_menos..],
        "después del horizonte"
    );
}
