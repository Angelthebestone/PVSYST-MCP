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
