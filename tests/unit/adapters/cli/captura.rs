use super::*;

#[test]
fn should_truncate_output_when_it_exceeds_the_byte_limit() {
    let mut captura = CapturaFlujo::default();
    let lleno = vec![b'x'; LIMITE_BYTES_STREAM];
    captura.agregar(&lleno);
    assert_eq!(captura.bytes.len(), LIMITE_BYTES_STREAM);
    assert!(!captura.truncado);

    let sobrante = vec![b'y'; 500];
    captura.agregar(&sobrante);
    assert_eq!(captura.bytes.len(), LIMITE_BYTES_STREAM);
    assert!(captura.truncado);
}
