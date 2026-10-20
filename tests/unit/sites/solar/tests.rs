use super::posicion::*;
use super::produccion::*;
use super::*;
use serde_json::json;

fn ecuador() -> Sitio {
    Sitio {
        latitud: 0.0,
        longitud: 0.0,
        zona_horaria: 0.0,
    }
}

fn colombia() -> Sitio {
    Sitio {
        latitud: 7.07,
        longitud: -73.84,
        zona_horaria: -5.0,
    }
}

#[test]
fn should_find_zenith_sun_at_solar_noon_on_equator() {
    let max = (0..1440)
        .map(|m| sol(&ecuador(), 80.0, f64::from(m) / 60.0).0)
        .fold(f64::MIN, f64::max);
    assert!(max > 89.5, "elevación máxima {max}");
}

#[test]
fn should_have_twelve_hours_of_sunlight_on_equator() {
    let horas_sol = (0..96)
        .filter(|q| sol(&ecuador(), 172.0, f64::from(*q) * 0.25).0 > 0.0)
        .count();
    let horas = horas_sol as f64 * 0.25;
    assert!((11.5..=12.5).contains(&horas), "horas de sol {horas}");
}

#[test]
fn should_have_east_azimuth_in_morning_and_west_in_afternoon() {
    let norte = Sitio {
        latitud: 40.0,
        longitud: 0.0,
        zona_horaria: 0.0,
    };
    assert!(sol(&norte, 172.0, 8.0).1 < 0.0);
    assert!(sol(&norte, 172.0, 16.0).1 > 0.0);
}

#[test]
fn should_point_north_at_noon_in_southern_hemisphere() {
    let sur = Sitio {
        latitud: -30.0,
        longitud: 0.0,
        zona_horaria: 0.0,
    };
    assert!(sol(&sur, 172.0, 12.0).1.abs() > 170.0);
}

#[test]
fn should_convert_valid_date_to_day_of_year() {
    assert_eq!(dia_del_anio("2021-06-21").ok(), Some(172));
    assert!(dia_del_anio("2021-02-30").is_err());
    assert!(dia_del_anio("21/06/2021").is_err());
}

#[test]
fn should_support_gregorian_leap_years_in_day_of_year() {
    assert_eq!(dia_del_anio("2024-02-29").ok(), Some(60));
    assert_eq!(dia_del_anio("2024-03-01").ok(), Some(61));
    assert_eq!(dia_del_anio("2023-03-01").ok(), Some(60));
    assert!(dia_del_anio("2023-02-29").is_err());
    assert!(dia_del_anio("1900-02-29").is_err());
    assert_eq!(dia_del_anio("2000-02-29").ok(), Some(60));
}

#[test]
fn should_receive_almost_all_horizontal_global_at_zero_tilt() {
    let clima = Clima::nuevo(vec![5.0; 12], vec![2.0; 12], vec![25.0; 12]).expect("12 meses");
    let plano = Orientacion {
        inclinacion: 0.0,
        azimut: 0.0,
    };
    let factor = factor_transposicion(&colombia(), &clima, plano);
    assert!(
        factor.iter().all(|f| (0.99..=1.01).contains(f)),
        "{factor:?}"
    );
}

#[test]
fn should_calculate_power_at_stc_with_system_losses() {
    let t_amb = 25.0 - 1000.0 * (NOCT - 20.0) / 800.0;
    assert!((energia_hora(1000.0, t_amb, 1.0) - 0.86).abs() < 1e-9);
}

#[test]
fn should_reject_clima_with_wrong_month_count() {
    assert!(Clima::nuevo(vec![5.0; 11], vec![2.0; 12], vec![25.0; 12]).is_err());
}

#[test]
fn should_reject_clima_with_nan_value() {
    let mut ghi = vec![5.0; 12];
    ghi[2] = f64::NAN;
    let res = Clima::nuevo(ghi, vec![2.0; 12], vec![20.0; 12]);
    assert!(res.is_err(), "debe fallar si GHI contiene NaN");
}

#[test]
fn should_reject_clima_with_negative_ghi() {
    let mut ghi = vec![5.0; 12];
    ghi[4] = -1.0;
    let res = Clima::nuevo(ghi, vec![2.0; 12], vec![20.0; 12]);
    assert!(res.is_err(), "debe fallar si GHI es negativo");
}

#[test]
fn should_reject_clima_when_diffuse_exceeds_global() {
    let ghi = vec![4.0; 12];
    let mut dif = vec![2.0; 12];
    dif[3] = 4.5;
    let res = Clima::nuevo(ghi, dif, vec![20.0; 12]);
    assert!(res.is_err(), "debe fallar si DIF > GHI");
}

#[test]
fn should_reject_clima_with_temperature_out_of_bounds() {
    let ghi = vec![5.0; 12];
    let dif = vec![2.0; 12];
    let mut temp = vec![20.0; 12];
    temp[0] = 85.0;
    assert!(Clima::nuevo(ghi.clone(), dif.clone(), temp).is_err());
    let mut temp_fria = vec![20.0; 12];
    temp_fria[0] = -120.0;
    assert!(Clima::nuevo(ghi, dif, temp_fria).is_err());
}

#[test]
fn should_reject_site_with_timezone_out_of_bounds() {
    let args_alta = json!({ "latitud": 10.0, "longitud": 10.0, "timezone": 20.0 });
    assert!(
        sitio_de(&args_alta).is_err(),
        "zona horaria 20 debe ser rechazada"
    );
    let args_baja = json!({ "latitud": 10.0, "longitud": 10.0, "timezone": -15.0 });
    assert!(
        sitio_de(&args_baja).is_err(),
        "zona horaria -15 debe ser rechazada"
    );
}

#[test]
fn should_reject_site_with_nan_coordinates_or_timezone() {
    let args_str_nan_lat = json!({ "latitud": "NaN", "longitud": 0.0 });
    assert!(sitio_de(&args_str_nan_lat).is_err());
    let args_null_lat = json!({ "latitud": f64::NAN, "longitud": 0.0 });
    assert!(sitio_de(&args_null_lat).is_err());
    let args_str_nan_tz = json!({ "latitud": 0.0, "longitud": 0.0, "timezone": "NaN" });
    assert!(sitio_de(&args_str_nan_tz).is_err());
    let args_null_tz = json!({ "latitud": 0.0, "longitud": 0.0, "timezone": f64::NAN });
    assert!(sitio_de(&args_null_tz).is_err());
}

#[test]
fn should_parse_valid_site_and_orientation() {
    let args = json!({ "latitud": 7.0, "longitud": -73.0, "timezone": -5.0 });
    let s = sitio_de(&args).expect("sitio válido");
    assert_eq!(s.zona_horaria, -5.0);
    assert_eq!(orientacion_de(&json!({}), 7.0).ok(), Some(None));
    let o = orientacion_de(&json!({"tilt": "10"}), -20.0).expect("válida");
    assert_eq!(
        o,
        Some(Orientacion {
            inclinacion: 10.0,
            azimut: 180.0
        })
    );
}
