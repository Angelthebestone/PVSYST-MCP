use super::*;

#[test]
fn should_find_containing_parent_object() {
    let texto = b"PVObject_Root=pvRoot\nPVObject_Hijo=pvChild\nX=1\nEnd of PVObject pvChild\nEnd of PVObject pvRoot\n";
    let spans = edit::spans_of(texto);
    let si = edit::find_span(&spans, "pvChild").expect("hijo");
    assert_eq!(padre_de(&spans, si).as_deref(), Some("pvRoot"));
}

#[test]
fn should_reject_variant_that_is_not_an_identifier_when_locating_it_beside_project() {
    let dir = tempfile::tempdir().expect("dir");
    let prj = dir.path().join("Planta.PRJ");
    std::fs::write(&prj, "x").expect("prj");
    std::fs::write(dir.path().join("x"), "x").expect("x");

    let res = crate::projects::anadir::ruta_variante(&prj, "VC0/../../x");

    assert!(matches!(res, Err(crate::error::Error::Invalid(_))));
}
