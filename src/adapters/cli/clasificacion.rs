//! Clasificación de resultados de ejecución de `PVsystCLI.exe`.
//!
//! Debido a que `PVsystCLI.exe` suele finalizar con código de salida 0 incluso
//! ante fallos, el resultado se determina analizando los marcadores en la salida estándar.
//! Cuando los marcadores son contradictorios o están ausentes donde se esperaba
//! confirmación, se clasifica explícitamente como ambiguo.

/// Substrings en la salida estándar que indican que el comando falló.
pub(crate) const ERROR_MARKERS: [&str; 22] = [
    "Invalid command syntax",
    "Unknown command",
    "Unknown option",
    "Required option",
    "expected a following",
    "Invalid value for option",
    "Invalid workspace path",
    "does not contain a workspace",
    "already open in another program",
    "cannot be run",
    "Logs cannot be exported",
    "A fatal error occurred",
    "Error when setting up environment",
    "is not writable",
    "Access is denied",
    "Acceso denegado",
    "Permission denied",
    "Cannot create file",
    "is missing",
    "was not specified",
    "already exists in the working directory",
    "Simulation cannot be run",
];

/// Clasificación explícita y tipada del resultado de un comando CLI.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub(crate) enum Clasificacion {
    Exito,
    Error(String),
    Ambiguo(String),
}

impl Clasificacion {
    pub(crate) fn etiqueta(&self) -> &'static str {
        match self {
            Self::Exito => "exito",
            Self::Error(_) => "error",
            Self::Ambiguo(_) => "ambiguo",
        }
    }
}

/// Líneas que PVsystCLI emite sobre licencias en cada ejecución (ruido informativo).
pub(crate) fn is_license_banner(line: &str) -> bool {
    let lower = line.to_ascii_lowercase();
    lower.starts_with("checking license")
        || lower.contains("license file")
        || lower.contains("lic-activate")
        || lower.contains("trial mode")
}

/// Subcadena positiva de confirmación requerida para cada comando de acción.
pub(crate) fn confirmed(command: &str, stdout: &str) -> bool {
    let needle = match command {
        "create-site" => "created with success",
        "convert-meteo" => "Conversion ended successfully",
        "run-simulation" => "Simulation done in ",
        "export-logs" => "Logs ZIP file successfully created",
        "lic-activate" => "successfully activated",
        "lic-sync" => "successfully synchronised",
        "lic-deactivate" => "successfully deactivated",
        _ => return false,
    };
    stdout.contains(needle)
}

/// Busca marcadores de error conocidos dentro del texto filtrado.
pub(crate) fn buscar_marcador_error(texto: &str) -> Option<String> {
    ERROR_MARKERS
        .iter()
        .find(|m| texto.contains(**m))
        .map(|m| (*m).to_string())
        .or_else(|| {
            texto
                .lines()
                .find(|l| {
                    (l.contains("Loading project") || l.contains("Loading workspace"))
                        && l.contains("failed")
                })
                .map(|l| l.trim().to_string())
        })
}

fn es_comando_informativo(command: &str) -> bool {
    matches!(command, "help" | "version" | "lic-info")
}

/// Clasifica la salida estándar de PVsystCLI considerando marcadores y confirmación.
///
/// Si existen marcadores contradictorios o si un comando de acción no emite
/// ni confirmación ni error, devuelve `Clasificacion::Ambiguo` en vez de adivinar.
pub(crate) fn clasificar(command: &str, stdout: &str) -> Clasificacion {
    let texto: String = stdout
        .lines()
        .filter(|l| !is_license_banner(l))
        .collect::<Vec<_>>()
        .join("\n");

    let error_detectado = buscar_marcador_error(&texto);
    let tiene_confirmacion = confirmed(command, &texto);

    if tiene_confirmacion && error_detectado.is_some() {
        let err = error_detectado.unwrap_or_default();
        return Clasificacion::Ambiguo(format!(
            "Marcadores contradictorios: confirmación de éxito y error ('{err}') en la salida"
        ));
    }

    if let Some(err) = error_detectado {
        return Clasificacion::Error(err);
    }

    if tiene_confirmacion {
        return Clasificacion::Exito;
    }

    if es_comando_informativo(command) {
        if texto.trim().is_empty() {
            return Clasificacion::Ambiguo(
                "Salida vacía o ausente para comando informativo".to_string(),
            );
        }
        return Clasificacion::Exito;
    }

    Clasificacion::Ambiguo(
        "Salida sin confirmación de éxito ni marcador de error concluyente".to_string(),
    )
}

/// Clasifica la salida del proceso teniendo en cuenta el código de salida si existe.
pub(crate) fn clasificar_salida(
    command: &str,
    stdout: &str,
    exit_code: Option<i32>,
) -> Clasificacion {
    if let Some(code) = exit_code {
        if code != 0 {
            return Clasificacion::Error(format!("Proceso terminó con código de error {code}"));
        }
    }
    clasificar(command, stdout)
}

/// Interfaz retrocompatible para llamadas externas que esperan `(bool, Option<String>)`.
pub(crate) fn classify(command: &str, stdout: &str) -> (bool, Option<String>) {
    match clasificar(command, stdout) {
        Clasificacion::Exito => (true, None),
        Clasificacion::Error(m) => (false, Some(m)),
        Clasificacion::Ambiguo(m) => (false, Some(format!("ambiguo: {m}"))),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const BANNER: &str =
        "Checking license...\r\nThe license file is missing.\r\nRunning in TRIAL mode\r\n";

    #[test]
    fn should_succeed_for_informative_commands_with_output() {
        assert_eq!(
            clasificar("help", include_str!("../../../tests/fixtures/cli/help.txt")),
            Clasificacion::Exito
        );
        assert_eq!(
            clasificar(
                "version",
                include_str!("../../../tests/fixtures/cli/version.txt")
            ),
            Clasificacion::Exito
        );
        assert_eq!(
            clasificar(
                "lic-info",
                include_str!("../../../tests/fixtures/cli/lic-info.txt")
            ),
            Clasificacion::Exito
        );
    }

    #[test]
    fn should_fail_for_syntax_and_missing_option_errors() {
        let unknown = clasificar(
            "nocommand",
            include_str!("../../../tests/fixtures/cli/unknown-command.txt"),
        );
        assert!(matches!(unknown, Clasificacion::Error(_)));
        let missing = clasificar(
            "create-site",
            include_str!("../../../tests/fixtures/cli/missing-option.txt"),
        );
        assert!(matches!(missing, Clasificacion::Error(_)));
        let bad = clasificar(
            "create-site",
            include_str!("../../../tests/fixtures/cli/bad-option.txt"),
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
        let site = include_str!("../../../tests/fixtures/cli/create-site-ok.txt");
        assert_eq!(clasificar("create-site", site), Clasificacion::Exito);
        let sim = include_str!("../../../tests/fixtures/cli/run-simulation-ok.txt");
        assert_eq!(clasificar("run-simulation", sim), Clasificacion::Exito);
        let ok = include_str!("../../../tests/fixtures/cli/export-logs-ok.txt");
        assert_eq!(clasificar("export-logs", ok), Clasificacion::Exito);
    }

    #[test]
    fn should_classify_nonzero_exit_code_as_error() {
        let res = clasificar_salida("help", "Some text", Some(1));
        assert!(matches!(res, Clasificacion::Error(_)));
    }

    #[test]
    fn should_maintain_backwards_compatible_classify_tuple() {
        let site = include_str!("../../../tests/fixtures/cli/create-site-ok.txt");
        assert!(classify("create-site", site).0);
        let bad = include_str!("../../../tests/fixtures/cli/bad-option.txt");
        assert!(!classify("create-site", bad).0);
    }
}
