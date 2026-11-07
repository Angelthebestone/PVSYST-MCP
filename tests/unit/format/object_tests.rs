use super::*;

#[test]
fn should_refuse_empty_and_truncated_files_when_reading() {
    let origen = Path::new("Demo.PRJ");
    assert!(matches!(parse_completo(" \n", origen), Err(Error::Format(m)) if m.contains("vacío")));
    let cortado = "PVObject_=pvSite\n  Site=Geneva\n";
    assert!(
        matches!(parse_completo(cortado, origen), Err(Error::Format(m)) if m.contains("truncado") && m.contains("pvSite"))
    );
    let completo = format!("{cortado}End of PVObject pvSite\n");
    assert_eq!(parse_completo(&completo, origen).unwrap().len(), 1);
    assert!(parse_completo("not a pvobject\n", origen)
        .unwrap()
        .is_empty());
}

#[test]
fn should_not_swallow_siblings_in_list_item() {
    let s = "PVObject_=pvVCalcul\n  Orientations, list of=1 TOrientGroup\n    PVObject_=pvOrient\n\
  Comment=Fixed\n    FieldTilt=25.0\n    End of TOrientGroup\n  End of Orientations\n\
PVObject_PVMainArray=pvMainArray\n    Flags=$00\n  End of PVObject pvMainArray\nEnd of PVObject pvVCalcul\n";
    let root = parse_first(s).expect("root");
    let types: Vec<&str> = root.children.iter().map(|c| c.obj_type.as_str()).collect();
    assert_eq!(types, ["pvOrient", "pvMainArray"]);
    assert_eq!(root.children[0].field("FieldTilt"), Some("25.0"));
}

#[test]
fn should_parse_bom_prefixed_document() {
    let s = "\u{FEFF}PVObject_=pvProject\r\n  Comment=Hi\r\nEnd of PVObject pvProject\r\n";
    let r = parse_first(s).expect("root despite BOM");
    assert_eq!(r.obj_type, "pvProject");
    assert_eq!(r.field("Comment"), Some("Hi"));
}
