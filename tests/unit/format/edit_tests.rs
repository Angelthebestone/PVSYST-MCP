use super::*;
use proptest::prelude::*;

const DOC: &str = "PVObject_=pvProject\r\n  Comment=Hello\r\n  \
PVObject_PVMainArray=pvMainArray\r\n    PVObject_=pvSubArray\r\n      \
PVModule=Old.PAN\r\n      NModSerie=10\r\n    End of PVObject pvSubArray\r\n  \
End of PVObject pvMainArray\r\nEnd of PVObject pvProject\r\n";

#[test]
fn should_reject_edit_when_selector_has_non_integer_index() {
    let err = edit_fields(
        DOC.as_bytes(),
        "pvSubArray[abc]",
        &[("NModSerie".into(), Some("20".into()))],
    )
    .unwrap_err();
    assert!(matches!(err, Error::Invalid(_)));
    assert!(err.to_string().contains("'pvSubArray[abc]'"));
}

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
    assert_eq!(outcomes[0].action, "replaced");
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
    assert_eq!(res[0].action, "replaced");
    let text = String::from_utf8(out).unwrap();
    assert!(text.contains("      PVModule=New.PAN\r\n"));
    assert!(!text.contains("Old.PAN"));
    assert!(text.contains("NModSerie=10\r\n"));
}

#[test]
fn should_insert_missing_field() {
    let (out, res) = edit_fields(
        DOC.as_bytes(),
        "pvSubArray",
        &[("NStringCh".into(), Some("48".into()))],
    )
    .unwrap();
    assert_eq!(res[0].action, "inserted");
    let text = String::from_utf8(out).unwrap();
    assert!(text.contains("      NStringCh=48\r\n"));
}

#[test]
fn should_keep_binary_bytes_identical() {
    let raw = b"PVObject_=pvProject\r\n  Comment=X\r\nEnd of PVObject pvProject\r\n\x8c\x00\x00\xff\xfe\r\n".to_vec();
    let (out, res) = edit_fields(&raw, "pvProject", &[]).unwrap();
    assert!(res.is_empty());
    assert_eq!(out, raw, "byte-identical when nothing changes");
}

#[test]
fn should_apply_every_object_edition_and_collect_their_outcomes() {
    let (out, outcomes) = editar_objetos(
        DOC.as_bytes(),
        &[
            EdicionObjeto {
                selector: "pvSubArray".into(),
                cambios: vec![("NModSerie".into(), Some("20".into()))],
            },
            EdicionObjeto {
                selector: "pvProject".into(),
                cambios: vec![("Comment".into(), Some("Edited".into()))],
            },
        ],
    )
    .expect("editar objetos");

    let texto = String::from_utf8(out).expect("utf8");
    assert!(texto.contains("NModSerie=20"));
    assert!(texto.contains("Comment=Edited"));
    assert_eq!(outcomes.len(), 2);
    assert_eq!(outcomes[0].key, "NModSerie");
    assert_eq!(outcomes[1].key, "Comment");
}

#[test]
fn should_keep_bytes_identical_when_there_are_no_object_editions() {
    let (out, outcomes) = editar_objetos(DOC.as_bytes(), &[]).expect("sin ediciones");
    assert_eq!(out, DOC.as_bytes());
    assert!(outcomes.is_empty());
}

proptest! {
    #[test]
    fn setting_a_field_changes_only_its_line_and_is_reversible(
        value in "[A-Za-z0-9_.,:-]{0,24}"
    ) {
        let (out, _) = edit_fields(
            DOC.as_bytes(),
            "pvSubArray",
            &[("NModSerie".into(), Some(value))],
        )
        .unwrap();
        let before: Vec<&[u8]> = DOC.as_bytes().split(|b| *b == b'\n').collect();
        let after: Vec<&[u8]> = out.split(|b| *b == b'\n').collect();
        prop_assert_eq!(before.len(), after.len());
        let changed = before.iter().zip(&after).filter(|(a, b)| a != b).count();
        prop_assert!(changed <= 1);

        let (back, _) = edit_fields(
            &out,
            "pvSubArray",
            &[("NModSerie".into(), Some("10".into()))],
        )
        .unwrap();
        prop_assert_eq!(back, DOC.as_bytes());
    }

    #[test]
    fn values_with_line_breaks_are_always_rejected(
        prefix in "[a-z]{0,5}",
        suffix in "[a-z]{0,5}",
        newline in prop::sample::select(vec!["\n", "\r", "\r\n"]),
    ) {
        let value = format!("{prefix}{newline}{suffix}");
        let result = edit_fields(
            DOC.as_bytes(),
            "pvSubArray",
            &[("NModSerie".into(), Some(value))],
        );
        prop_assert!(result.is_err());
    }
}
