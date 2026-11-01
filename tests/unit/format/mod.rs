use super::*;
use proptest::prelude::*;

#[test]
fn value_text_leaves_strings_unquoted() {
    assert_eq!(value_text(&Value::String("a b".into())), "a b");
    assert_eq!(value_text(&serde_json::json!(2.5)), "2.5");
}

proptest! {
    #[test]
    fn rounding_is_idempotent_and_close(x in -1.0e6f64..1.0e6, d in 0i32..4) {
        let r = redondear(x, d);
        prop_assert_eq!(redondear(r, d), r);
        prop_assert!((r - x).abs() <= 0.5 * 10f64.powi(-d) + 1e-9);
    }
}

#[test]
fn should_read_numbers_with_decimal_comma_or_point() {
    assert_eq!(numero_texto(" 1,5 "), Some(1.5));
    assert_eq!(numero_texto("-2.25"), Some(-2.25));
    assert_eq!(numero_json(&serde_json::json!(3)), Some(3.0));
    assert_eq!(numero_json(&serde_json::json!("4,0")), Some(4.0));
}

#[test]
fn should_refuse_non_finite_or_non_numeric_text() {
    for t in ["NaN", "inf", "-infinity", "", "abc", "1.2.3"] {
        assert_eq!(numero_texto(t), None, "{t}");
    }
    assert_eq!(numero_json(&serde_json::json!(true)), None);
}

#[test]
fn should_round_trip_latin1_and_refuse_wider_characters() {
    assert_eq!(codificar_latin1("Español"), Some(b"Espa\xf1ol".to_vec()));
    assert_eq!(decodificar_latin1(b"Espa\xf1ol"), "Español");
    assert_eq!(codificar_latin1("€"), None);
}
