use super::*;
use proptest::prelude::*;

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

const SAMPLE: &str = "PVObject_=pvProject\n  Comment=Hello\n  \
PVObject_SitePrj=pvSite\n    Site=Geneva\n  End of PVObject pvSite\n\
End of PVObject pvProject\n";

#[test]
fn should_parse_nested_tree() {
    let root = parse_first(SAMPLE).expect("root");
    assert_eq!(root.obj_type, "pvProject");
    assert_eq!(root.field("Comment"), Some("Hello"));
    assert_eq!(root.find_deep("Site"), Some("Geneva"));
    assert_eq!(root.children.len(), 1);
    assert_eq!(root.children[0].role.as_deref(), Some("SitePrj"));
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
fn should_detect_pvobject() {
    assert!(is_pvobject(SAMPLE));
    assert!(!is_pvobject("pvModule;Soitec\tX;\nVersion 5;\n"));
}

#[test]
fn should_parse_bom_prefixed_document() {
    let s = "\u{FEFF}PVObject_=pvProject\r\n  Comment=Hi\r\nEnd of PVObject pvProject\r\n";
    let r = parse_first(s).expect("root despite BOM");
    assert_eq!(r.obj_type, "pvProject");
    assert_eq!(r.field("Comment"), Some("Hi"));
}

#[test]
fn should_diagnose_balanced_document() {
    let diag = diagnosticar_estructura(SAMPLE);
    assert!(diag.esta_balanceada());
    assert!(diag.abiertos_sin_cerrar.is_empty());
    assert_eq!(diag.cierres_sin_apertura, 0);
}

#[test]
fn should_diagnose_unclosed_objects_in_truncated_document() {
    let trunc =
        "PVObject_=pvProject\n  Comment=Hello\n  PVObject_SitePrj=pvSite\n    Site=Geneva\n";
    let diag = diagnosticar_estructura(trunc);
    assert!(!diag.esta_balanceada());
    assert_eq!(diag.abiertos_sin_cerrar, vec!["pvProject", "pvSite"]);
    assert_eq!(diag.cierres_sin_apertura, 0);

    // Read continues returning roots despite truncation.
    let roots = parse(trunc);
    assert_eq!(roots.len(), 2);
}

#[test]
fn should_diagnose_closing_without_opening() {
    let extra_close = "PVObject_=pvProject\nEnd of PVObject pvProject\nEnd of PVObject extra\n";
    let diag = diagnosticar_estructura(extra_close);
    assert!(!diag.esta_balanceada());
    assert_eq!(diag.cierres_sin_apertura, 1);
}

#[test]
fn should_validate_structure_accepting_balanced_and_rejecting_truncated() {
    assert!(validar_estructura(SAMPLE.as_bytes()).is_ok());

    let trunc = "PVObject_=pvProject\n  Comment=Hello\n";
    let err = validar_estructura(trunc.as_bytes()).unwrap_err();
    assert!(matches!(err, Error::Format(_)));
    assert!(err.to_string().contains("1 objeto(s) sin cerrar"));
    assert!(err.to_string().contains("[pvProject]"));
}
#[test]
fn should_accept_synthetic_variant_with_implicit_list_and_type_closures() {
    let synthetic_vc = "\
PVObject_=pvVCalcul
  Orientations, list of=1 TOrientGroup
    PVObject_=pvOrient
      Comment=Fixed
      FieldTilt=25.0
    End of TOrientGroup
  End of Orientations
  PVObject_Ageing=pvAgeing
    AgeingLossPure=TCubicProfile
  End of TAgeing
  VarList, list of=1 TVarSimul
    End of VarSimul=viHSol
  End of the List VarList
End of PVObject pvVCalcul
";
    let diag = diagnosticar_estructura(synthetic_vc);
    assert!(
        diag.esta_balanceada(),
        "unclosed: {:?}, extra closes: {}",
        diag.abiertos_sin_cerrar,
        diag.cierres_sin_apertura
    );
    assert!(validar_estructura(synthetic_vc.as_bytes()).is_ok());

    let trunc = synthetic_vc
        .trim_end()
        .strip_suffix("End of PVObject pvVCalcul")
        .unwrap();
    let err = validar_estructura(trunc.as_bytes()).unwrap_err();
    assert!(matches!(err, Error::Format(_)));
    assert!(err.to_string().contains("[pvVCalcul]"));
}

proptest! {
    #[test]
    fn parsing_arbitrary_text_never_panics(text in "(?s).{0,400}") {
        let _ = parse(&text);
        let _ = parse_first(&text);
        let _ = diagnosticar_estructura(&text);
    }

    #[test]
    fn parsing_structured_noise_never_panics(
        lines in prop::collection::vec(
            prop::sample::select(vec![
                "PVObject_=pvX",
                "PVObject_Role=pvY",
                "End of PVObject pvX",
                "  Key=Value",
                "EndTags",
                "/",
                "  List, of=2 TItem",
                "  End of TItem",
            ]),
            0..40,
        )
    ) {
        let text = lines.join("\n");
        let _ = parse(&text);
        let _ = diagnosticar_estructura(&text);
    }
}
