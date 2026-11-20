//! Behaviour of the shared Gregorian calendar arithmetic.

use super::*;
use proptest::prelude::*;

proptest! {
    #[test]
    fn should_roundtrip_day_counts_when_converting_between_civil_and_day_counts(
        dias in -1_000_000i64..1_000_000,
    ) {
        prop_assert_eq!(dias_desde_civil(civil_desde_dias(dias)), dias);
    }
}
