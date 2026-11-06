use super::*;

#[test]
fn should_handle_unclosed_pvorient_absorbing_group_keys() {
    let texto = b"PVObject_=pvVCalcul\n  PVObject_=pvOrient\n    FieldAzim=0.0\n    FieldTilt=10.0\n  FieldTilt=25.0\n  FieldAzim=20.0\nEnd of PVObject pvVCalcul\n";
    let dups = buscar_duplicados(texto);
    assert_eq!(dups.len(), 1);
    assert_eq!(dups[0].0, "pvOrient[0]");
    assert!(dups[0].1.contains("FieldAzim: 0.0 | 20.0"));
    assert!(dups[0].1.contains("FieldTilt: 10.0 | 25.0"));
}
