use super::*;
use std::time::Duration;

/// The line PVsystCLI 8.1.8 wrote when a `run-simulation` died (measured).
const LOG_CAIDA: &str = "01:58:59|Err |EPrivilege      |run-simulation      |Instruction privilégiée\n        |    |                |                    |1.110 sec;\n";

fn log_dir(contenido: &str) -> tempfile::TempDir {
    let dir = tempfile::tempdir().expect("carpeta de logs temporal");
    std::fs::write(dir.path().join("20261108_015857Err.log"), contenido).expect("log de prueba");
    dir
}

#[test]
fn should_report_the_exception_pvsystcli_logged() {
    let dir = log_dir(LOG_CAIDA);
    let desde = SystemTime::now() - Duration::from_secs(1);
    let crash = crash_en_directorio(dir.path(), desde);
    assert_eq!(crash.as_deref(), Some("Instruction privilégiée"));
}

#[test]
fn should_ignore_a_log_written_before_the_run_started() {
    let dir = log_dir(LOG_CAIDA);
    let desde = SystemTime::now() + Duration::from_secs(60);
    assert_eq!(crash_en_directorio(dir.path(), desde), None);
}

#[test]
fn should_ignore_a_log_without_a_crash() {
    let dir = log_dir("01:59:08|Warn|TEnvManager     |InitLicense         |No license file\n");
    let desde = SystemTime::now() - Duration::from_secs(1);
    assert_eq!(crash_en_directorio(dir.path(), desde), None);
}
