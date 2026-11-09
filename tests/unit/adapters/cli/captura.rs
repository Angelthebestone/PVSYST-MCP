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

/// PVsystCLI writes in the console code page: an accented error marker must survive
/// decoding, or `Acceso denegado` never matches and the failure reads as ambiguous.
#[test]
fn should_decode_a_console_code_page_byte_instead_of_dropping_it() {
    let sink = crear_sink();
    // 0xE1 is `á` in CP1252 and not valid UTF-8.
    sink.lock()
        .expect("sink libre")
        .agregar(b"Acceso denegado: ruta no v\xE1lida");
    assert_eq!(
        texto_sink(&sink).expect("texto"),
        "Acceso denegado: ruta no válida"
    );
}
