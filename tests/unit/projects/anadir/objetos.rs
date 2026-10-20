use super::*;

#[test]
fn should_find_containing_parent_object() {
    let texto = b"PVObject_Root=pvRoot\nPVObject_Hijo=pvChild\nX=1\nEnd of PVObject pvChild\nEnd of PVObject pvRoot\n";
    let spans = edit::spans_of(texto);
    let si = edit::find_span(&spans, "pvChild").expect("hijo");
    assert_eq!(padre_de(&spans, si).as_deref(), Some("pvRoot"));
}
