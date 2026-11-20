use super::*;

const SAMPLE: &str = "Lat Lon Jan Feb Mar Apr May Jun Jul Aug Sep Oct Nov Dec Ann\n-90 -180 9.0 8.0 7.0 6.0 5.0 4.0 3.0 2.0 1.0 2.0 3.0 4.0 4.5\n7 -74 5.34 5.34 5.28 4.96 5.01 5.16 5.57 5.55 5.27 4.80 4.72 4.86 5.16\n";

#[test]
fn should_read_twelve_monthly_values_when_the_grid_cell_exists() {
    let v = parse_grid_at(SAMPLE, 7, -74)
        .expect("lectura exitosa")
        .expect("celda presente");
    assert_eq!(v.len(), 12);
    assert!((v[0] - 5.34).abs() < 1e-9);
    assert!((v[11] - 4.86).abs() < 1e-9);
}

#[test]
fn should_reject_the_row_when_a_monthly_value_is_unparseable() {
    let sample_con_abc = "Lat Lon Jan Feb Mar Apr May Jun Jul Aug Sep Oct Nov Dec Ann\n7 -74 5.34 abc 5.28 4.96 5.01 5.16 5.57 5.55 5.27 4.80 4.72 4.86 5.16\n";
    let err = parse_grid_at(sample_con_abc, 7, -74)
        .expect_err("un valor no numérico no puede convertirse en 0")
        .to_string();
    assert!(err.contains("abc"), "debe reportar 'abc': {err}");
    assert!(
        err.contains("7") && err.contains("-74"),
        "debe reportar la celda 7/-74: {err}"
    );
}
