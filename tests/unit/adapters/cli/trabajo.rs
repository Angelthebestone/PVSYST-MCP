use super::*;

#[test]
fn should_parse_progress_percentage_with_newlines() {
    assert_eq!(
        progreso("Simulation 07/02/90 - 10%\nSimulation 07/02/90 - 35%"),
        Some(35.0)
    );
    assert_eq!(progreso("sin progreso"), None);
}

#[test]
fn should_parse_progress_percentage_with_carriage_returns() {
    assert_eq!(
        progreso("Simulation 07/02/90 - 10%\rSimulation 07/02/90 - 80%\r"),
        Some(80.0)
    );
    assert_eq!(progreso("sin progreso\r"), None);
}

#[test]
fn should_record_error_instead_of_running_forever_when_job_panics() {
    let res = sin_panico(|| panic!("fallo inesperado"));

    assert!(matches!(res, Err(crate::error::Error::Cli(m)) if m.contains("pánico")));
}

#[test]
fn should_pass_through_the_job_result_when_it_does_not_panic() {
    let res = sin_panico(|| Err(crate::error::Error::Cli("PVsystCLI ausente".into())));

    assert!(matches!(res, Err(crate::error::Error::Cli(m)) if m == "PVsystCLI ausente"));
}

#[test]
fn should_not_launch_job_when_the_jobs_lock_is_poisoned() {
    let ws = tempfile::tempdir().expect("ws");
    let paths = Paths::para_test(ws.path());
    let estado = std::sync::Arc::clone(&paths.estado);
    let _ = std::thread::spawn(move || {
        let _guard = estado.trabajos.lock();
        panic!("envenenar");
    })
    .join();

    let res = run_en_segundo_plano(&paths, &["run-simulation".into()], Duration::from_secs(1));

    assert!(matches!(res, Err(crate::error::Error::Cli(m)) if m.contains("envenenado")));
}
