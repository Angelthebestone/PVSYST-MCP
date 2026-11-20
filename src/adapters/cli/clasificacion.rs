//! Clasificación de resultados de ejecución de `PVsystCLI.exe`.
//!
//! Debido a que `PVsystCLI.exe` suele finalizar con código de salida 0 incluso
//! ante fallos, el resultado se determina analizando los marcadores en la salida estándar.
//! Cuando los marcadores son contradictorios o están ausentes donde se esperaba
//! confirmación, se clasifica explícitamente como ambiguo.

use crate::idioma::{Idioma, Texto};

const PROCESO_CON_ERROR: Texto = Texto {
    es: "Proceso terminó con código de error {code}",
    en: "Process ended with error code {code}",
};
const SALIDA_VACIA: Texto = Texto {
    es: "Salida vacía para '{command}'",
    en: "Empty output for '{command}'",
};
const SIN_CONFIRMACION: Texto = Texto {
    es: "Salida sin confirmación de éxito ni marcador de error concluyente",
    en: "Output without a success confirmation or a conclusive error marker",
};
const MARCADORES_CONTRADICTORIOS: Texto = Texto {
    es: "Marcadores contradictorios: confirmación de éxito y error ('{err}') en la salida",
    en: "Contradictory markers: success confirmation and error ('{err}') in the output",
};
const SALIDA_INFORMATIVA_VACIA: Texto = Texto {
    es: "Salida vacía o ausente para comando informativo",
    en: "Empty or missing output for an informative command",
};

/// Substrings en la salida estándar que indican que el comando falló.
pub(crate) const ERROR_MARKERS: [&str; 24] = [
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
    // Measured in 8.1.8: `export-logs` to a folder that does not exist.
    "does not exist. Please set another target folder",
    // Measured in 8.1.8: `convert-meteo` with a CSV whose dates are unreadable.
    "Invalid date Line",
];

/// Comienzos de línea que PVsystCLI imprime solo cuando rechazó el comando. Se
/// buscan al principio de la línea para no confundirlos con el texto de ayuda,
/// que documenta esas mismas cadenas. Medidos en 8.1.8 (rev. 49527):
/// `Error: Invalid command syntax`, `Unknown command line option : --x`,
/// `Unknown option : x` y `Unknown command "help"`.
const PREFIJOS_ERROR_EXPLICITO: [&str; 3] = [
    "Error: Invalid command syntax",
    "Unknown command",
    "Unknown option :",
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
/// Los comandos de PVsystCLI no distinguen mayúsculas de minúsculas.
const CONFIRMACIONES: [(&str, &str); 7] = [
    ("create-site", "created with success"),
    ("convert-meteo", "Conversion ended successfully"),
    ("run-simulation", "Simulation done in "),
    ("export-logs", "Logs ZIP file successfully created"),
    ("lic-activate", "successfully activated"),
    ("lic-sync", "successfully synchronised"),
    ("lic-deactivate", "successfully deactivated"),
];

pub(crate) fn confirmed(command: &str, stdout: &str) -> bool {
    // Informative commands (`help`, `version`, `lic-info`) have no success marker:
    // a non-empty answer without an explicit error is their confirmation.
    if es_comando_informativo(command) {
        return !stdout.trim().is_empty() && error_explicito(stdout).is_none();
    }
    CONFIRMACIONES
        .iter()
        .find(|(nombre, _)| command.eq_ignore_ascii_case(nombre))
        .is_some_and(|(_, needle)| stdout.contains(needle))
}

/// Primer error explícito de la línea de comandos en `texto`, si lo hay.
fn error_explicito(texto: &str) -> Option<String> {
    texto
        .lines()
        .map(str::trim)
        .find(|l| PREFIJOS_ERROR_EXPLICITO.iter().any(|p| l.starts_with(p)))
        .map(str::to_string)
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

pub(crate) fn es_comando_informativo(command: &str) -> bool {
    ["help", "version", "lic-info"]
        .iter()
        .any(|c| command.eq_ignore_ascii_case(c))
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
        return Clasificacion::Ambiguo(
            MARCADORES_CONTRADICTORIOS.texto_con(Idioma::de_entorno(), &[("err", &err)]),
        );
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
                SALIDA_INFORMATIVA_VACIA
                    .en_idioma(Idioma::de_entorno())
                    .to_string(),
            );
        }
        return Clasificacion::Exito;
    }

    Clasificacion::Ambiguo(SIN_CONFIRMACION.en_idioma(Idioma::de_entorno()).to_string())
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
        Some(code) if code != 0 => Clasificacion::Error(
            PROCESO_CON_ERROR.texto_con(Idioma::de_entorno(), &[("code", &code.to_string())]),
        ),
        _ => clasificar(command, stdout),
    }
}

fn clasificar_informativo(command: &str, stdout: &str) -> Clasificacion {
    // El atajo por código de salida existe porque el texto de ayuda documenta
    // marcadores como "Required option"; un error explícito sigue siendo un error.
    if let Some(err) = error_explicito(stdout) {
        return Clasificacion::Error(err);
    }
    if stdout.trim().is_empty() {
        return Clasificacion::Ambiguo(
            SALIDA_VACIA.texto_con(Idioma::de_entorno(), &[("command", command)]),
        );
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
