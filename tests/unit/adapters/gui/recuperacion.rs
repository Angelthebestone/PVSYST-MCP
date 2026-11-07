//! Escalation of `reiniciar`: a crash dialog's button first, a forced kill only
//! when the process survives it, and an honest answer when even that fails.

use super::{asegurar_muerte, Muerte};

#[test]
fn should_not_force_a_kill_when_the_process_ended_after_the_button() {
    let muerte = asegurar_muerte(
        || false,
        || panic!("a dead process must not be killed"),
        || panic!("nothing is left to wait for"),
    );

    assert_eq!(muerte, Muerte::Termino);
    assert_eq!(muerte.nombre(), "termino");
}

#[test]
fn should_force_a_kill_when_the_button_does_not_end_the_process() {
    let muerte = asegurar_muerte(|| true, || true, || false);

    assert_eq!(muerte, Muerte::Matado);
    assert_eq!(muerte.nombre(), "kill_forzado");
}

#[test]
fn should_report_the_process_when_it_survives_the_forced_kill() {
    assert_eq!(
        asegurar_muerte(|| true, || true, || true),
        Muerte::SigueVivo
    );
}

#[test]
fn should_report_the_process_when_windows_refuses_the_kill() {
    let muerte = asegurar_muerte(
        || true,
        || false,
        || panic!("without a kill there is nothing to wait for"),
    );

    assert_eq!(muerte, Muerte::SigueVivo);
}
