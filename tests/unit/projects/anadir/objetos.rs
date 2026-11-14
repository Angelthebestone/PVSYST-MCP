#[test]
fn should_reject_variant_that_is_not_an_identifier_when_locating_it_beside_project() {
    let dir = tempfile::tempdir().expect("dir");
    let prj = dir.path().join("Planta.PRJ");
    std::fs::write(&prj, "x").expect("prj");
    std::fs::write(dir.path().join("x"), "x").expect("x");

    let res = crate::projects::anadir::ruta_variante(&prj, "VC0/../../x");

    assert!(matches!(res, Err(crate::error::Error::Invalid(_))));
}
