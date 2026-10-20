use super::*;

#[test]
fn should_truncate_output_when_exceeding_byte_limit() {
    let mut captura = CapturaFlujo::default();
    let trozo = vec![b'a'; LIMITE_BYTES_STREAM + 100];
    captura.agregar(&trozo);
    assert_eq!(captura.bytes.len(), LIMITE_BYTES_STREAM);
    assert!(captura.truncado);
}

#[test]
fn should_keep_reading_without_growing_beyond_limit() {
    let mut captura = CapturaFlujo::default();
    let trozo = vec![b'x'; LIMITE_BYTES_STREAM];
    captura.agregar(&trozo);
    assert_eq!(captura.bytes.len(), LIMITE_BYTES_STREAM);
    assert!(!captura.truncado);

    let mas = vec![b'y'; 500];
    captura.agregar(&mas);
    assert_eq!(captura.bytes.len(), LIMITE_BYTES_STREAM);
    assert!(captura.truncado);
}

#[test]
fn should_surface_error_when_mutex_is_poisoned() {
    let sink = crear_sink();
    let sink_clon = Arc::clone(&sink);
    let _ = thread::spawn(move || {
        if let Ok(_guard) = sink_clon.lock() {
            panic!("pánico forzado para envenenar el mutex");
        }
    })
    .join();

    assert!(texto_sink(&sink).is_err());
    assert!(es_truncado(&sink).is_err());
}

#[test]
fn should_surface_error_when_reader_thread_panics() {
    let handle: JoinHandle<Result<()>> = thread::spawn(|| {
        panic!("pánico en hilo de lectura");
    });
    let join_res = handle
        .join()
        .map_err(|_| Error::Cli("Hilo de lectura terminó con pánico".to_string()));
    assert!(join_res.is_err());
}
