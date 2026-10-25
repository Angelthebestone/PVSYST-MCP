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

/// Clasifica la salida del proceso. Con código de salida, los comandos informativos
/// (`help`, `version`, `lic-info`) se deciden solo por el código: su texto de ayuda
/// documenta marcadores como "Required option" sin ser un error.
pub(crate) fn clasificar_salida(
    command: &str,
    stdout: &str,
    exit_code: Option<i32>,
) -> Clasificacion {
    match exit_code {
        Some(0) if es_comando_informativo(command) => clasificar_informativo(command, stdout),
        Some(code) if code != 0 => {
            Clasificacion::Error(format!("Proceso terminó con código de error {code}"))
        }
        _ => clasificar(command, stdout),
    }
}

fn clasificar_informativo(command: &str, stdout: &str) -> Clasificacion {
    if stdout.trim().is_empty() {
        return Clasificacion::Ambiguo(format!("Salida vacía para '{command}'"));
    }
    Clasificacion::Exito
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
#[path = "../../../tests/unit/adapters/cli/clasificacion.rs"]
mod tests;
