use super::*;
use crate::idioma::Idioma;
use serde_json::json;

#[test]
fn should_name_the_meteonorm_cause_and_the_hourly_meteo_alternative() {
    // Given: PVsyst's own words for a simulation that could not start
    let salida = json!({
        "success": false,
        "detected_error": "Simulation cannot be run",
        "stdout_tail": "Failed to initialize Meteonorm\nDataMN9/station.db.enc",
    });

    // When
    let detalle = explicar(&salida, Idioma::En).expect("causa conocida");

    // Then
    assert!(detalle.contains("Meteonorm 9"), "{detalle}");
    assert!(detalle.contains("hourly .MET"), "{detalle}");
    assert!(
        detalle.contains("synthetic_weather does not replace"),
        "{detalle}"
    );
}

#[test]
fn should_explain_nothing_when_the_output_mentions_no_known_failure() {
    let salida = json!({ "success": false, "stdout_tail": "Simulation done in 0min 2sec" });
    assert_eq!(explicar(&salida, Idioma::En), None);
    assert_eq!(explicar(&json!({}), Idioma::En), None);
}
