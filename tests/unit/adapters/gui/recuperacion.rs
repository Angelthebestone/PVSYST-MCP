//! Escalation of `reiniciar`: the answer must be honest when even a forced kill
//! leaves the process alive.

use super::{asegurar_muerte, Muerte};

#[test]
fn should_report_the_process_when_it_survives_the_forced_kill() {
    assert_eq!(
        asegurar_muerte(|| true, || true, || true),
        Muerte::SigueVivo
    );
}
