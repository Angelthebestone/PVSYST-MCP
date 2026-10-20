use super::*;

#[test]
fn should_warn_when_generic_component_has_no_file() {
    let aviso = aviso_no_resuelto("Generic_Mono_300W.PAN", "no encontrado");
    assert!(aviso.contains("componente interno de PVsyst"));
    assert!(aviso.starts_with("Generic_Mono_300W.PAN"));
}
