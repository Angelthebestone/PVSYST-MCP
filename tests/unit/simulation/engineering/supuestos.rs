use super::*;
use serde_json::json;

#[test]
fn should_reject_non_finite_out_of_range_and_fractional_assumptions() {
    assert!(acotado(&json!({"degradacion": 1.5}), &DEGRADACION, None).is_err());
    assert!(acotado(&json!({"degradacion": -0.1}), &DEGRADACION, None).is_err());
    assert!(acotado(&json!({}), &DEGRADACION, None).is_err());
    assert_eq!(
        acotado(&json!({}), &DEGRADACION, Some(0.01)).ok(),
        Some(0.01)
    );
    assert!(anios(&json!({"anios": 2.5}), "anios", 25.0).is_err());
    assert!(anios(&json!({"anios": 1e9}), "anios", 25.0).is_err());
    assert_eq!(anios(&json!({}), "anios", 25.0).ok(), Some(25));
}
