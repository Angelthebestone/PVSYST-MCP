use super::*;

#[test]
fn should_record_error_instead_of_running_forever_when_job_panics() {
    let res = sin_panico(|| panic!("fallo inesperado"));

    assert!(matches!(res, Err(crate::error::Error::Cli(m)) if m.contains("pánico")));
}
