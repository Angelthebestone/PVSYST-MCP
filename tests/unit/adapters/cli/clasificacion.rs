use super::*;

const BANNER: &str =
    "Checking license...\r\nThe license file is missing.\r\nRunning in TRIAL mode\r\n";

#[test]
fn should_classify_as_ambiguous_when_confirmation_and_error_contradict() {
    let salida = "Simulation done in 0min 3sec\r\nA fatal error occurred\r\n";
    let res = clasificar("run-simulation", salida);
    assert!(matches!(res, Clasificacion::Ambiguo(_)));
}

#[test]
fn should_not_mask_error_marker_with_license_banner() {
    let stdout = format!("{BANNER}Access is denied\r\n");
    let res = clasificar("export-logs", &stdout);
    assert!(matches!(res, Clasificacion::Error(_)));
}
