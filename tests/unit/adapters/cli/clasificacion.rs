use super::*;

const BANNER: &str =
    "Checking license...\r\nThe license file is missing.\r\nRunning in TRIAL mode\r\n";

#[test]
fn should_succeed_for_informative_commands_with_output() {
    assert_eq!(
        clasificar(
            "help",
            include_str!(concat!(
                env!("CARGO_MANIFEST_DIR"),
                "/tests/fixtures/cli/help.txt"
            ))
        ),
        Clasificacion::Exito
    );
    assert_eq!(
        clasificar(
            "version",
            include_str!(concat!(
                env!("CARGO_MANIFEST_DIR"),
                "/tests/fixtures/cli/version.txt"
            ))
        ),
        Clasificacion::Exito
    );
    assert_eq!(
        clasificar(
            "lic-info",
            include_str!(concat!(
                env!("CARGO_MANIFEST_DIR"),
                "/tests/fixtures/cli/lic-info.txt"
            ))
        ),
        Clasificacion::Exito
    );
}

#[test]
fn should_fail_for_syntax_and_missing_option_errors() {
    let unknown = clasificar(
        "nocommand",
        include_str!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/tests/fixtures/cli/unknown-command.txt"
        )),
    );
    assert!(matches!(unknown, Clasificacion::Error(_)));
    let missing = clasificar(
        "create-site",
        include_str!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/tests/fixtures/cli/missing-option.txt"
        )),
    );
    assert!(matches!(missing, Clasificacion::Error(_)));
    let bad = clasificar(
        "create-site",
        include_str!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/tests/fixtures/cli/bad-option.txt"
        )),
    );
    assert!(matches!(bad, Clasificacion::Error(_)));
}

#[test]
fn should_classify_as_ambiguous_when_confirmation_and_error_contradict() {
    let salida = "Simulation done in 0min 3sec\r\nA fatal error occurred\r\n";
    let res = clasificar("run-simulation", salida);
    assert!(matches!(res, Clasificacion::Ambiguo(_)));
}

#[test]
fn should_classify_as_ambiguous_when_confirmation_absent_for_action_command() {
    let salida = "Proceso ejecutado sin marcador conocido\r\n";
    let res = clasificar("run-simulation", salida);
    assert!(matches!(res, Clasificacion::Ambiguo(_)));
}

#[test]
fn should_classify_as_ambiguous_when_informative_output_is_empty() {
    let salida = format!("{BANNER}\r\n");
    let res = clasificar("help", &salida);
    assert!(matches!(res, Clasificacion::Ambiguo(_)));
}

#[test]
fn should_not_mask_error_marker_with_license_banner() {
    let stdout = format!("{BANNER}Access is denied\r\n");
    let res = clasificar("export-logs", &stdout);
    assert!(matches!(res, Clasificacion::Error(_)));
}

#[test]
fn should_succeed_for_valid_action_fixtures() {
    let site = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/tests/fixtures/cli/create-site-ok.txt"
    ));
    assert_eq!(clasificar("create-site", site), Clasificacion::Exito);
    let sim = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/tests/fixtures/cli/run-simulation-ok.txt"
    ));
    assert_eq!(clasificar("run-simulation", sim), Clasificacion::Exito);
    let ok = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/tests/fixtures/cli/export-logs-ok.txt"
    ));
    assert_eq!(clasificar("export-logs", ok), Clasificacion::Exito);
}

#[test]
fn should_classify_nonzero_exit_code_as_error() {
    let res = clasificar_salida("help", "Some text", Some(1));
    assert!(matches!(res, Clasificacion::Error(_)));
}
#[test]
fn should_classify_help_with_exit_zero_as_success_despite_option_text() {
    let salida = "run-simulation: Required option -p <project> is needed";
    let res = clasificar_salida("help", salida, Some(0));
    assert_eq!(res, Clasificacion::Exito);
}

#[test]
fn should_maintain_backwards_compatible_classify_tuple() {
    let site = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/tests/fixtures/cli/create-site-ok.txt"
    ));
    assert!(classify("create-site", site).0);
    let bad = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/tests/fixtures/cli/bad-option.txt"
    ));
    assert!(!classify("create-site", bad).0);
}
