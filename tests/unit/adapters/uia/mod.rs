#[test]
fn should_preserve_com_error_when_reading_string() {
    let mut errores = Vec::new();
    let valor =
        super::arbol::leer_str_prop(Err(windows::core::Error::empty()), "nombre", &mut errores);
    assert!(valor.is_null());
    assert_eq!(errores.len(), 1);
    assert!(errores[0].starts_with("nombre:"));
}
