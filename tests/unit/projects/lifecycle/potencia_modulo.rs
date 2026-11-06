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
