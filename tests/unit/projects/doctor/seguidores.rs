use super::*;

fn variante(extra: &str) -> String {
    format!(
        "PVObject_=pvVCalcul\n  PVObject_=pvOrient\n    FieldType=UnlimTracking\n{extra}  End of TOrientGroup\nEnd of PVObject pvVCalcul\n"
    )
}

#[test]
fn should_report_error_when_pitch_does_not_exceed_collector_width() {
    let mut informe = Informe::default();
    revisar_seguidor(
        &variante("    Pitch=4.000\n    CollectorWidth=4.250\n    MinPhi=-60.0\n    MaxPhi=60.0\n"),
        &mut informe,
    );
    assert_eq!(informe.errores.len(), 1);
    assert!(informe.errores[0].contains("Pitch"));
}

#[test]
fn should_ignore_fixed_plane_variants() {
    let mut informe = Informe::default();
    revisar_seguidor(
        "PVObject_=pvVCalcul\n  PVObject_=pvOrient\n    FieldType=FixedPlane\n    FieldTilt=25.0\n  End of TOrientGroup\nEnd of PVObject pvVCalcul\n",
        &mut informe,
    );
    assert!(informe.errores.is_empty());
    assert!(informe.avisos.is_empty());
}
