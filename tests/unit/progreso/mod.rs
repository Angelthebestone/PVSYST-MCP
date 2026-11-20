//! Behaviour of the neutral task tracker: cancellation state and reporting.

use super::*;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};

#[test]
fn should_report_progress_and_expose_cancellation_when_a_sink_is_attached() {
    let cancelado = Arc::new(AtomicBool::new(false));
    let vistos = Arc::new(Mutex::new(Vec::new()));
    let destino = Arc::clone(&vistos);
    let seguimiento = SeguimientoTarea::nuevo(
        Arc::clone(&cancelado),
        Some(Box::new(move |porcentaje: f64, mensaje: &str| {
            destino
                .lock()
                .expect("sin envenenamiento")
                .push((porcentaje, mensaje.to_string()));
        })),
    );

    assert!(!seguimiento.esta_cancelado());
    seguimiento.informar_progreso(10.0, "Simulando");
    cancelado.store(true, Ordering::Relaxed);

    assert!(seguimiento.esta_cancelado());
    assert_eq!(
        *vistos.lock().expect("sin envenenamiento"),
        vec![(10.0, "Simulando".to_string())]
    );
}
