use super::*;

#[test]
fn should_map_known_and_unknown_control_types() {
    assert_eq!(rol_de(50000), "button");
    assert_eq!(rol_de(50019), "tab item");
    assert_eq!(rol_de(1), "otro");
}

#[test]
fn should_preserve_com_error_when_reading_string() {
    let mut errores = Vec::new();
    let valor =
        super::arbol::leer_str_prop(Err(windows::core::Error::empty()), "nombre", &mut errores);
    assert!(valor.is_null());
    assert_eq!(errores.len(), 1);
    assert!(errores[0].starts_with("nombre:"));
}
