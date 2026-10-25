use super::*;
use serde_json::json;

#[test]
fn should_honour_esperar_segundos_over_timeout_ms() {
    let args = json!({"esperar_segundos": 2, "timeout_ms": 500});
    assert_eq!(duracion_ms(&args), 2_000);
}

#[test]
fn should_use_timeout_ms_when_no_seconds_are_given() {
    let args = json!({"timeout_ms": 500});
    assert_eq!(duracion_ms(&args), 500);
}

#[test]
fn should_default_when_neither_duration_is_given() {
    assert_eq!(duracion_ms(&json!({})), DEFAULT_TIMEOUT_MS);
}
