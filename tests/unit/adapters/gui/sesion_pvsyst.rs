use super::debe_recogerse;

/// Pid of the PVsyst the live run left orphaned (2026-11-06 findings report).
const PID: u32 = 7488;

#[test]
fn should_close_the_pvsyst_this_call_started_when_the_step_fails() {
    // Given: this session started PVsyst and it is still running
    let vivos = [PID];

    // When
    let a_cerrar = debe_recogerse(Some(PID), &vivos);

    // Then
    assert_eq!(a_cerrar, Some(PID));
}

#[test]
fn should_leave_a_pvsyst_that_was_already_running_untouched() {
    // Given: PVsyst was open before the call (`ya_en_ejecucion`)
    let vivos = [PID];

    // When
    let a_cerrar = debe_recogerse(None, &vivos);

    // Then
    assert_eq!(a_cerrar, None);
}
