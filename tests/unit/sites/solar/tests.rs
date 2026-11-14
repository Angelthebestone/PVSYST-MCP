use super::posicion::dia_del_anio;
use super::*;

#[test]
fn should_support_gregorian_leap_years_in_day_of_year() {
    assert_eq!(dia_del_anio("2024-02-29").ok(), Some(60));
    assert_eq!(dia_del_anio("2024-03-01").ok(), Some(61));
    assert_eq!(dia_del_anio("2023-03-01").ok(), Some(60));
    assert!(matches!(
        dia_del_anio("2023-02-29"),
        Err(crate::error::Error::Invalid(_))
    ));
    assert!(matches!(
        dia_del_anio("1900-02-29"),
        Err(crate::error::Error::Invalid(_))
    ));
    assert_eq!(dia_del_anio("2000-02-29").ok(), Some(60));
}

#[test]
fn should_reject_clima_when_diffuse_exceeds_global() {
    let ghi = vec![4.0; 12];
    let mut dif = vec![2.0; 12];
    dif[3] = 4.5;
    let res = Clima::nuevo(ghi, dif, vec![20.0; 12]);
    assert!(
        matches!(res, Err(crate::error::Error::Invalid(_))),
        "debe fallar si DIF > GHI"
    );
}
