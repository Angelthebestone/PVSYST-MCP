//! Behaviour of the neutral task tracker: cancellation state and reporting.

use super::*;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};

#[test]
fn should_stay_active_and_silent_when_inert() {
    let seguimiento = SeguimientoTarea::inerte();
    assert!(!seguimiento.esta_cancelado());
    seguimiento.informar_progreso(50.0, "Simulando");
}

#[test]
fn should_report_progress_and_expose_cancellation() {
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
