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
    let sink = crear_sink();
    let lector: JoinHandle<Result<()>> = thread::spawn(|| panic!("pánico en hilo de lectura"));

    let res = esperar_lector(
        lector,
        (&sink, "stdout"),
        Instant::now() + Duration::from_secs(60),
    );

    assert!(matches!(res, Err(Error::Cli(m)) if m.contains("stdout") && m.contains("pánico")));
}

#[test]
fn should_return_captured_output_marked_truncated_when_pipe_never_closes() {
    let sink = crear_sink();
    let (tx, rx) = std::sync::mpsc::channel::<()>();
    let lector: JoinHandle<Result<()>> = thread::spawn(move || {
        let _ = rx.recv();
        Ok(())
    });

    let res = esperar_lector(lector, (&sink, "stdout"), Instant::now());

    assert!(res.is_ok());
    assert!(es_truncado(&sink).expect("sink"));
    drop(tx);
}

#[test]
fn should_propagate_read_error_of_the_reader_thread() {
    let sink = crear_sink();
    let lector: JoinHandle<Result<()>> = thread::spawn(|| {
        Err(Error::io(
            "Error leyendo pipe de stderr",
            std::io::Error::other("roto"),
        ))
    });

    let res = esperar_lector(
        lector,
        (&sink, "stderr"),
        Instant::now() + Duration::from_secs(60),
    );

    assert!(matches!(res, Err(Error::Io { .. })));
    assert!(!es_truncado(&sink).expect("sink"));
}
