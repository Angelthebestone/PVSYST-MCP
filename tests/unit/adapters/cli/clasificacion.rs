use super::*;

const BANNER: &str =
    "Checking license...\r\nThe license file is missing.\r\nRunning in TRIAL mode\r\n";

#[test]
fn should_not_mask_error_marker_with_license_banner() {
    let stdout = format!("{BANNER}Access is denied\r\n");
    let res = clasificar("export-logs", &stdout);
    assert!(matches!(res, Clasificacion::Error(_)));
}
