//! A UIA call that never returns must not block the server (GUI-02).

use super::*;
use std::sync::mpsc;

#[test]
fn should_stop_waiting_for_a_call_that_does_not_return() {
    let (_libera, bloqueada) = mpsc::channel::<()>();

    let resultado = con_limite(Duration::from_millis(100), move || bloqueada.recv().ok())
        .expect("a blocked call is a timeout, not an error");

    assert_eq!(resultado, None);
}

#[test]
fn should_return_what_a_call_that_finishes_in_time_produced() {
    let resultado = con_limite(Duration::from_secs(5), || 42).expect("the thread starts");

    assert_eq!(resultado, Some(42));
}
