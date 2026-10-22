use super::*;
use serde_json::json;

const SIGMA_TEST: Rango = Rango {
    clave: "sigma_test",
    min: 0.0,
    max: 100.0,
};

#[test]
fn should_reject_non_finite_out_of_range_assumptions() {
    assert!(acotado(&json!({"sigma_test": 150.0}), &SIGMA_TEST, None).is_err());
    assert!(acotado(&json!({"sigma_test": -0.1}), &SIGMA_TEST, None).is_err());
    assert!(acotado(&json!({}), &SIGMA_TEST, None).is_err());
    assert_eq!(acotado(&json!({}), &SIGMA_TEST, Some(5.0)).ok(), Some(5.0));
    assert_eq!(
        acotado(&json!({"sigma_test": 10.0}), &SIGMA_TEST, Some(5.0)).ok(),
        Some(10.0)
    );
}
