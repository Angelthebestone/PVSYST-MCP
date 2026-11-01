use super::leer_variante;
use std::fs;
use tempfile::tempdir;

#[test]
fn should_name_the_variant_candidates_when_the_variant_file_is_missing() {
    let dir = tempdir().expect("dir");
    fs::write(dir.path().join("Plant.VC1"), "x").expect("variante");
    let err = leer_variante(&dir.path().join("Plant.VC0"), dir.path()).expect_err("no existe");
    let texto = err.to_string();
    assert!(
        texto.contains("Variante 'Plant.VC0' no encontrado"),
        "{texto}"
    );
    assert!(texto.contains("Plant.VC1"), "{texto}");
    assert!(!texto.contains("os error"), "{texto}");
}

#[test]
fn should_refuse_a_truncated_saved_variant() {
    let truncada = b"PVObject_=pvVCalcul\r\n  PVObject_=pvSubArray\r\n    NModSerie=18\r\n";

    let res = super::raiz_completa(truncada, std::path::Path::new("Planta.VC0"));

    assert!(matches!(res, Err(crate::error::Error::Format(m)) if m.contains("truncado")));
}
