#[test]
fn should_refuse_a_truncated_saved_variant() {
    let truncada = b"PVObject_=pvVCalcul\r\n  PVObject_=pvSubArray\r\n    NModSerie=18\r\n";

    let res = super::raiz_completa(truncada, std::path::Path::new("Planta.VC0"));

    assert!(matches!(res, Err(crate::error::Error::Format(m)) if m.contains("truncado")));
}
