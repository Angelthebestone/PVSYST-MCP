use super::*;
use serde_json::json;

#[test]
fn should_honour_esperar_segundos_over_timeout_ms() {
    let args = json!({"esperar_segundos": 2, "timeout_ms": 500});
    assert_eq!(duracion_ms(&args), 2_000);
}
