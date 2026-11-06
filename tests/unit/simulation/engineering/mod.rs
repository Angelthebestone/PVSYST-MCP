use super::incertidumbre::incertidumbre;

#[test]
fn should_combine_sigmas_when_reporting_p90_and_p99() {
    let v = incertidumbre(1000.0, 5.0, 3.0, "defecto");
    assert!((v["p90_kwh"].as_f64().unwrap() - 925.3).abs() < 0.5);
    assert!((v["p99_kwh"].as_f64().unwrap() - 864.4).abs() < 0.5);
}
