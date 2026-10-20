use super::*;

#[test]
fn should_compute_pnom_from_sizing_dividing_kwp_by_modules() {
    let text = "PVObject_=pvSubArray
  PNomSizing=9.0
  NModSerie=15
  NStringCh=2
End of PVObject pvSubArray
";
    let root = object::parse_first(text).expect("parse");
    assert_eq!(pnom_from_sizing(&root), Some(300.0));
}

#[test]
fn should_extract_pnom_from_name_with_watt_suffix() {
    assert_eq!(pnom_from_name("Generic_Mono_300W.PAN"), Some(300.0));
    assert_eq!(pnom_from_name("CX-M500_2450Wp_1_00_086.PAN"), None);
    assert_eq!(pnom_from_name("Module.PAN"), None);
}
