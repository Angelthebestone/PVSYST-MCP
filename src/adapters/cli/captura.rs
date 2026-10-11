//! Captura de salidas estándar (stdout y stderr) con límite de memoria.
//!
//! Evita que un subproceso bloquee sus pipes mientras protege al servidor MCP
//! de un consumo descontrolado de memoria cuando el proceso emite logs masivos.

use crate::error::{Error, Result};
pub(crate) use crate::state::{CapturaFlujo, Sink};
use std::io::Read;
use std::sync::{Arc, Mutex};
use std::thread::{self, JoinHandle};

/// Límite máximo de bytes almacenados por flujo (stdout o stderr).
/// Si el subproceso supera este límite, los bytes restantes se siguen
/// drenando para evitar bloqueos del pipe, pero no se acumulan en memoria.
pub(crate) const LIMITE_BYTES_STREAM: usize = 1024 * 1024; // 1 MiB

/// Tamaño del buffer temporal para lecturas de pipe.
const TAMANO_BUFFER_PIPE: usize = 8192;

impl CapturaFlujo {
    /// Agrega un fragmento de bytes respetando el límite máximo asignado.
    pub(crate) fn agregar(&mut self, trozo: &[u8]) {
        if self.bytes.len() < LIMITE_BYTES_STREAM {
            let espacio_libre = LIMITE_BYTES_STREAM - self.bytes.len();
            let a_copiar = trozo.len().min(espacio_libre);
            self.bytes.extend_from_slice(&trozo[..a_copiar]);
            if trozo.len() > espacio_libre {
                self.truncado = true;
            }
        } else {
            self.truncado = true;
        }
    }
}

pub(crate) fn crear_sink() -> Sink {
    Arc::new(Mutex::new(CapturaFlujo::default()))
}

/// Drena continuamente un pipe en un hilo independiente hasta alcanzar EOF.
///
/// Si la lectura falla por I/O o el mutex se corrompe, se propaga el error
/// en lugar de fallar silenciosamente con salida vacía.
pub(crate) fn drain<R: Read + Send + 'static>(
    pipe: Option<R>,
    sink: Sink,
    nombre_flujo: &'static str,
) -> JoinHandle<Result<()>> {
    thread::spawn(move || {
        let Some(mut pipe) = pipe else {
            return Ok(());
        };
        let mut buf = [0u8; TAMANO_BUFFER_PIPE];
        loop {
            match pipe.read(&mut buf) {
                Ok(0) => break,
                Ok(n) => {
                    let mut guard = sink.lock().map_err(|_| {
                        Error::Cli(format!("Mutex envenenado en captura de {nombre_flujo}"))
                    })?;
                    guard.agregar(&buf[..n]);
                }
                Err(e) if e.kind() == std::io::ErrorKind::Interrupted => continue,
                Err(e) => {
                    if let Ok(mut guard) = sink.lock() {
                        guard.error_io = Some(e.to_string());
                    }
                    return Err(Error::io(
                        format!("Error leyendo pipe de {nombre_flujo}"),
                        e,
                    ));
                }
            }
        }
        Ok(())
    })
}

/// Extrae el texto acumulado del sink, propagando si el mutex está envenenado.
pub(crate) fn texto_sink(sink: &Sink) -> Result<String> {
    let guard = sink
        .lock()
        .map_err(|_| Error::Cli("Mutex de captura de salida envenenado".to_string()))?;
    Ok(String::from_utf8_lossy(&guard.bytes).into_owned())
}

/// Consulta si el flujo alcanzó el límite de bytes y fue truncado.
pub(crate) fn es_truncado(sink: &Sink) -> Result<bool> {
    let guard = sink
        .lock()
        .map_err(|_| Error::Cli("Mutex de captura de salida envenenado".to_string()))?;
    Ok(guard.truncado)
}

#[cfg(test)]
mod tests {
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
}
