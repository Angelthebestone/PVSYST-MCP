use super::*;

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
fn should_reject_project_stem_that_leaves_the_folder_when_building_variant_route() {
    let parent = Path::new("proj");
    for stem in ["..\\..\\X", "../X", "\\Users\\X", "C:X", ""] {
        assert!(
            matches!(ruta_variante(parent, stem, "VC0"), Err(Error::Invalid(_))),
            "{stem}"
        );
    }
}
