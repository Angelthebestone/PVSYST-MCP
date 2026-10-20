use super::*;

#[test]
fn should_detect_duplicated_keys_in_configuration() {
    let texto = b"PVObject_Root=pvOrient\nFieldTilt=10\nFieldTilt=20\nEnd of PVObject pvOrient\n";
    let dups = buscar_duplicados(texto);
    assert_eq!(dups.len(), 1);
    assert_eq!(dups[0].0, "pvOrient[0]");
    assert!(dups[0].1.contains("FieldTilt"));
}

#[test]
fn should_handle_unclosed_pvorient_absorbing_group_keys() {
    let texto = b"PVObject_=pvVCalcul\n  PVObject_=pvOrient\n    FieldAzim=0.0\n    FieldTilt=10.0\n  FieldTilt=25.0\n  FieldAzim=20.0\nEnd of PVObject pvVCalcul\n";
    let dups = buscar_duplicados(texto);
    assert_eq!(dups.len(), 1);
    assert_eq!(dups[0].0, "pvOrient[0]");
    assert!(dups[0].1.contains("FieldAzim: 0.0 | 20.0"));
    assert!(dups[0].1.contains("FieldTilt: 10.0 | 25.0"));
}

#[test]
fn should_ignore_result_types_and_contd_keys() {
    let texto =
        b"PVObject_Root=pvSimulPlot\nFieldTilt=10\nFieldTilt=20\nEnd of PVObject pvSimulPlot\n";
    assert!(buscar_duplicados(texto).is_empty());
}

#[test]
fn should_ignore_identical_values_and_contd() {
    let texto = b"PVObject_Root=pvSystem\nFlags=1\nFlags=1\nX_contd=2\nX_contd=3\nEnd of PVObject pvSystem\n";
    assert!(buscar_duplicados(texto).is_empty());
}

#[test]
fn should_report_clean_when_no_duplicates_exist() {
    let texto = b"PVObject_Root=pvOrient\nFieldTilt=10\nEnd of PVObject pvOrient\n";
    assert!(buscar_duplicados(texto).is_empty());
}
