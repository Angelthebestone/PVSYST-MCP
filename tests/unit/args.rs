use super::*;
use serde_json::json;

#[test]
fn should_tell_a_missing_number_from_a_non_numeric_one() {
    assert_eq!(req_numero(&json!({ "x": " 4.5 " }), "x", "").unwrap(), 4.5);
    assert!(matches!(
        req_numero(&json!({}), "x", ""),
        Err(Error::MissingArg { .. })
    ));
    let err = req_numero(&json!({ "x": "abc" }), "x", "").unwrap_err();
    assert!(matches!(&err, Error::Invalid(m) if m.contains("'x'") && m.contains("abc")));
    assert!(matches!(
        req_numero(&json!({ "x": "inf" }), "x", ""),
        Err(Error::Invalid(_))
    ));
}

#[test]
fn should_name_the_key_when_an_optional_number_is_out_of_range() {
    let error = rango_opcional(Some(10.5), 10.0, "tasa").expect_err("fuera de rango");

    assert!(error.to_string().contains("'tasa'"), "{error}");
    assert!(matches!(
        rango_opcional(Some(-0.1), 10.0, "tasa"),
        Err(crate::error::Error::Invalid(_))
    ));
}
