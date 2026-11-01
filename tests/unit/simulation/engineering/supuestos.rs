use super::*;
use serde_json::json;

const SIGMA_TEST: Rango = Rango {
    clave: "sigma_test",
    min: 0.0,
    max: 100.0,
};

#[test]
fn should_reject_non_finite_out_of_range_assumptions() {
    assert!(matches!(
        acotado(&json!({"sigma_test": 150.0}), &SIGMA_TEST, None),
        Err(crate::error::Error::Invalid(_))
    ));
    assert!(matches!(
        acotado(&json!({"sigma_test": -0.1}), &SIGMA_TEST, None),
        Err(crate::error::Error::Invalid(_))
    ));
    assert!(matches!(
        acotado(&json!({}), &SIGMA_TEST, None),
        Err(crate::error::Error::MissingArg { .. } | crate::error::Error::MissingArgs { .. })
    ));
    assert_eq!(acotado(&json!({}), &SIGMA_TEST, Some(5.0)).ok(), Some(5.0));
    assert_eq!(
        acotado(&json!({"sigma_test": 10.0}), &SIGMA_TEST, Some(5.0)).ok(),
        Some(10.0)
    );
}
