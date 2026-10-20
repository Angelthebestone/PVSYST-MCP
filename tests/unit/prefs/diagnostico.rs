use super::*;

#[test]
fn resumir_log_keeps_only_non_norm_levels() {
    let texto = "12:00:00|Norm|mod|fn|ok\n12:00:01|Err|mod|fn|fallo\n";
    let resumen = resumir_log(Path::new("a.log"), texto, FiltroLog::SoloErrores);
    let no_norm = resumen["lineas_no_norm"].as_array().unwrap();
    assert_eq!(no_norm.len(), 1);
    assert!(resumen.get("ultimas").is_none());
}
