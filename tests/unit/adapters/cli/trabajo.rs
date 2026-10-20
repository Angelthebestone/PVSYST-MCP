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
