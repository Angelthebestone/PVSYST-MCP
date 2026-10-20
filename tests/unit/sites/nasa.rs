use super::*;

const SAMPLE: &str = "Lat Lon Jan Feb Mar Apr May Jun Jul Aug Sep Oct Nov Dec Ann\n-90 -180 9.0 8.0 7.0 6.0 5.0 4.0 3.0 2.0 1.0 2.0 3.0 4.0 4.5\n7 -74 5.34 5.34 5.28 4.96 5.01 5.16 5.57 5.55 5.27 4.80 4.72 4.86 5.16\n";

#[test]
fn should_find_cell_and_return_twelve_values() {
    let v = parse_grid_at(SAMPLE, 7, -74)
        .expect("lectura exitosa")
        .expect("celda presente");
    assert_eq!(v.len(), 12);
    assert!((v[0] - 5.34).abs() < 1e-9);
    assert!((v[11] - 4.86).abs() < 1e-9);
}

#[test]
fn should_return_none_when_cell_is_missing() {
    assert!(parse_grid_at(SAMPLE, 10, 10).expect("lectura").is_none());
}

#[test]
fn should_reject_row_with_unparseable_monthly_value() {
    let sample_con_abc = "Lat Lon Jan Feb Mar Apr May Jun Jul Aug Sep Oct Nov Dec Ann\n7 -74 5.34 abc 5.28 4.96 5.01 5.16 5.57 5.55 5.27 4.80 4.72 4.86 5.16\n";
    let res = parse_grid_at(sample_con_abc, 7, -74);
    assert!(res.is_err(), "debe fallar con valor no numérico");
    let err = res.unwrap_err().to_string();
    assert!(err.contains("abc"), "debe reportar 'abc': {err}");
    assert!(
        err.contains("7") && err.contains("-74"),
        "debe reportar celda 7/-74"
    );
}

#[test]
fn should_reject_row_with_nan_monthly_value() {
    let sample_con_nan = "Lat Lon Jan Feb Mar Apr May Jun Jul Aug Sep Oct Nov Dec Ann\n7 -74 5.34 NaN 5.28 4.96 5.01 5.16 5.57 5.55 5.27 4.80 4.72 4.86 5.16\n";
    let res = parse_grid_at(sample_con_nan, 7, -74);
    assert!(res.is_err(), "debe fallar con NaN");
}

#[test]
fn should_calculate_monthly_totals_using_days_per_month() {
    let daily = vec![1.0; 12];
    let m = monthly_totals(&daily);
    assert_eq!(m[0], 31.0);
    assert_eq!(m[1], 28.0);
    assert_eq!(m[11], 31.0);
}
