//! Behaviour of the shared Gregorian calendar arithmetic.

use super::*;
use proptest::prelude::*;

#[test]
fn should_map_known_day_counts() {
    let epoch = civil_desde_dias(0);
    assert_eq!((epoch.anio(), epoch.mes(), epoch.dia()), (1970, 1, 1));

    let leap = civil_desde_dias(11_016);
    assert_eq!((leap.anio(), leap.mes(), leap.dia()), (2000, 2, 29));
}

#[test]
fn should_know_leap_years_and_month_lengths() {
    assert!(es_bisiesto(2000));
    assert!(es_bisiesto(2024));
    assert!(!es_bisiesto(1900));
    assert!(!es_bisiesto(2023));
    assert_eq!(dias_del_mes(2024, 2), 29);
    assert_eq!(dias_del_mes(2023, 2), 28);
    assert_eq!(dias_del_mes(2023, 4), 30);
    assert_eq!(dias_del_mes(2023, 13), 0);
}

proptest! {
    #[test]
    fn should_roundtrip_day_counts(dias in -1_000_000i64..1_000_000) {
        prop_assert_eq!(dias_desde_civil(civil_desde_dias(dias)), dias);
    }

    #[test]
    fn should_roundtrip_valid_dates(
        (anio, mes, dia) in (-2000i64..4000, 1u32..=12).prop_flat_map(|(a, m)| {
            (Just(a), Just(m), 1u32..=dias_del_mes(a, m))
        }),
    ) {
        let fecha = FechaCivil::nuevo(anio, mes, dia);
        let back = civil_desde_dias(dias_desde_civil(fecha));
        prop_assert_eq!((back.anio(), back.mes(), back.dia()), (anio, mes, dia));
    }
}
