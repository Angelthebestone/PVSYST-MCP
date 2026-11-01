use super::*;

#[test]
fn operacion_para_picks_the_natural_pattern() {
    assert_eq!(operacion_para(Some("Checkbox")), "alternar");
    assert_eq!(operacion_para(Some("tab item")), "seleccionar");
    assert_eq!(operacion_para(Some("tree item")), "expandir");
    assert_eq!(operacion_para(Some("button")), "invocar");
    assert_eq!(operacion_para(None), "invocar");
}
