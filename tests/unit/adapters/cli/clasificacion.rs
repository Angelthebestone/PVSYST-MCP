use super::*;

const BANNER: &str =
    "Checking license...\r\nThe license file is missing.\r\nRunning in TRIAL mode\r\n";

#[test]
fn should_not_mask_error_marker_with_license_banner() {
    let stdout = format!("{BANNER}Access is denied\r\n");
    let res = clasificar("export-logs", &stdout);
    assert!(matches!(res, Clasificacion::Error(_)));
}

/// `help --bogus` prints the syntax error and still exits with code 0, so the
/// informative shortcut must not turn it into a success.
#[test]
fn should_report_explicit_error_of_informative_command() {
    let casos = [
        (
            "help",
            "\r\nError: Invalid command syntax\r\n\r\nUnknown command line option : --bogus\r\n",
        ),
        ("help", "Unknown command \"help\"\r\n"),
        (
            "version",
            "\r\nError: Invalid command syntax\r\n\r\nUnknown command line option : --bogus\r\n",
        ),
        (
            "lic-info",
            "\r\nError: Invalid command syntax\r\n\r\nUnknown command line option : --bogus\r\n",
        ),
    ];
    for (comando, stdout) in casos {
        let res = clasificar_salida(comando, stdout, Some(0));
        assert!(
            matches!(res, Clasificacion::Error(_)),
            "{comando} {stdout:?} -> {res:?}"
        );
    }
}

/// The CLI documents case-insensitive commands, so `VERSION` and `Help` are successes.
#[test]
fn should_accept_informative_command_in_any_case() {
    for (comando, stdout) in [
        ("VERSION", "PVsystCLI 8.1.8 (rev. 49527)\r\n"),
        (
            "Help",
            "Available commands:\r\n   version: Show PVsystCLI version\r\n",
        ),
    ] {
        let res = clasificar_salida(comando, stdout, Some(0));
        assert_eq!(res, Clasificacion::Exito, "{comando}");
    }
}

/// Messages PVsystCLI prints instead of a confirmation when the action failed.
#[test]
fn should_report_clear_text_error_of_action_command() {
    let casos = [
        (
            "export-logs",
            "C:/Users/x/QA_tmp_logs does not exist. Please set another target folder.\r\n",
        ),
        (
            "convert-meteo",
            "Running weather data conversion ...\r\nInvalid date Line 6.\r\n",
        ),
    ];
    for (comando, stdout) in casos {
        let res = clasificar(comando, stdout);
        assert!(
            matches!(res, Clasificacion::Error(_)),
            "{comando} {stdout:?} -> {res:?}"
        );
    }
}
