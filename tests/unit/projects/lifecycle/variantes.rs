use super::*;

#[test]
fn should_validate_variant_identifier_format() {
    assert!(es_variante_valida("VC0"));
    assert!(es_variante_valida("cm9"));
    assert!(es_variante_valida("VCA"));
    assert!(!es_variante_valida("VC"));
    assert!(!es_variante_valida("VC10"));
    assert!(!es_variante_valida("XX1"));
    assert!(!es_variante_valida("VC.."));
    assert!(!es_variante_valida("V/1"));
}

#[test]
fn should_reject_path_escape_in_variant_route() {
    let parent = Path::new("proj");
    assert!(matches!(
        ruta_variante(parent, "Planta", "..\\x"),
        Err(crate::error::Error::Invalid(_))
    ));
    assert_eq!(
        ruta_variante(parent, "Planta", "VC1").expect("valid"),
        parent.join("Planta.VC1")
    );
}

#[test]
fn should_parse_only_positive_integers() {
    assert_eq!(parse_entero(&json!(15)), Some(15));
    assert_eq!(parse_entero(&json!("15")), Some(15));
    assert_eq!(parse_entero(&json!("1a")), None);
    assert_eq!(parse_entero(&json!("")), None);
    assert_eq!(parse_entero(&json!(0)), None);
    assert_eq!(parse_entero(&json!(2.5)), None);
}

#[test]
fn should_reject_project_stem_that_leaves_the_folder_when_building_variant_route() {
    let parent = Path::new("proj");
    for stem in ["..\\..\\X", "../X", "\\Users\\X", "C:X", ""] {
        assert!(
            matches!(ruta_variante(parent, stem, "VC0"), Err(Error::Invalid(_))),
            "{stem}"
        );
    }
}
