use super::*;

const DOC: &str = "PVObject_=pvProject\r\n  Comment=Hello\r\n  \
PVObject_PVMainArray=pvMainArray\r\n    PVObject_=pvSubArray\r\n      \
PVModule=Old.PAN\r\n      NModSerie=10\r\n    End of PVObject pvSubArray\r\n  \
End of PVObject pvMainArray\r\nEnd of PVObject pvProject\r\n";

#[test]
fn should_edit_second_instance_when_selector_has_index_one() {
    let multi = "PVObject_=pvRoot\r\n  \
PVObject_=pvSubArray\r\n    NMod=10\r\n  End of PVObject pvSubArray\r\n  \
PVObject_=pvSubArray\r\n    NMod=20\r\n  End of PVObject pvSubArray\r\n\
End of PVObject pvRoot\r\n";
    let (out, outcomes) = edit_fields(
        multi.as_bytes(),
        "pvSubArray[1]",
        &[("NMod".into(), Some("99".into()))],
    )
    .expect("edit second instance");
    assert_eq!(outcomes[0].action, AccionCambio::Replaced);
    assert_eq!(outcomes[0].old_value.as_deref(), Some("20"));

    let text = String::from_utf8(out).expect("utf8");
    assert!(text.contains("    NMod=10\r\n"));
    assert!(text.contains("    NMod=99\r\n"));
}

#[test]
fn should_refuse_edit_when_pvobject_is_truncated() {
    let truncated = "PVObject_=pvProject\r\n  Comment=Hello\r\n";
    let err = edit_fields(
        truncated.as_bytes(),
        "pvProject",
        &[("Comment".into(), Some("New".into()))],
    )
    .unwrap_err();
    assert!(matches!(err, Error::Format(_)));
    assert!(err.to_string().contains("sin cerrar"));
}

#[test]
fn should_replace_field_at_right_level() {
    let (out, res) = edit_fields(
        DOC.as_bytes(),
        "pvMainArray/pvSubArray",
        &[("PVModule".into(), Some("New.PAN".into()))],
    )
    .unwrap();
    assert_eq!(res[0].action, AccionCambio::Replaced);
    let text = String::from_utf8(out).unwrap();
    assert!(text.contains("      PVModule=New.PAN\r\n"));
    assert!(!text.contains("Old.PAN"));
    assert!(text.contains("NModSerie=10\r\n"));
}

#[test]
fn should_keep_binary_bytes_identical() {
    let raw = b"PVObject_=pvProject\r\n  Comment=X\r\nEnd of PVObject pvProject\r\n\x8c\x00\x00\xff\xfe\r\n".to_vec();
    let (out, res) = edit_fields(&raw, "pvProject", &[]).unwrap();
    assert!(res.is_empty());
    assert_eq!(out, raw, "byte-identical when nothing changes");
}

#[test]
fn should_report_sin_cambio_and_keep_the_bytes_when_the_value_is_already_set() {
    let (out, outcomes) = edit_fields(
        DOC.as_bytes(),
        "pvSubArray",
        &[("NModSerie".into(), Some("10".into()))],
    )
    .expect("el valor ya era ese");
    assert_eq!(outcomes[0].action, AccionCambio::SinCambio);
    assert_eq!(out, DOC.as_bytes());
}
