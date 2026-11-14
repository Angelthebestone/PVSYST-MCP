/// BUG-05: the variant was read right after «Guardar», half written or still unchanged.
#[test]
fn should_wait_for_the_variant_to_change_and_stop_changing() {
    let antes = b"viejo";

    assert!(!super::es_guardado_estable(antes, None, b"viejo"));
    assert!(!super::es_guardado_estable(antes, Some(b"viejo"), b"viejo"));
    assert!(!super::es_guardado_estable(antes, None, b"nuevo a med"));
    assert!(!super::es_guardado_estable(
        antes,
        Some(b"nuevo a med"),
        b"nuevo completo"
    ));
    assert!(super::es_guardado_estable(
        antes,
        Some(b"nuevo completo"),
        b"nuevo completo"
    ));
}

#[test]
fn should_refuse_a_truncated_saved_variant() {
    let truncada = b"PVObject_=pvVCalcul\r\n  PVObject_=pvSubArray\r\n    NModSerie=18\r\n";

    let res = super::raiz_completa(truncada, std::path::Path::new("Planta.VC0"));

    assert!(matches!(res, Err(crate::error::Error::Format(m)) if m.contains("truncado")));
}
