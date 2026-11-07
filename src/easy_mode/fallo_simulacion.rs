//! Why a simulation could not run, in the words the caller needs. PVsyst
//! reports the failure in its own terms (`Failed to initialize Meteonorm…`)
//! and leaves the caller to work out what to change.

use crate::idioma::{Idioma, Texto};
use serde_json::Value;

/// A failure PVsyst prints, with the change that gets past it.
struct Fallo {
    /// Text PVsyst prints for this failure.
    marca: &'static str,
    /// What actually happened.
    causa: Texto,
    /// What the caller can do instead.
    alternativa: Texto,
}

const FALLOS: &[Fallo] = &[Fallo {
    marca: "Failed to initialize Meteonorm",
    causa: Texto {
        es: "PVsyst genera la meteorología horaria sintética con Meteonorm 9 y no pudo inicializarlo",
        en: "PVsyst generates the synthetic hourly weather with Meteonorm 9 and could not initialise it",
    },
    alternativa: Texto {
        es: "usa un sitio que ya tenga un .MET horario (sitio meteo_pvgis o convertir_meteo): synthetic_weather no sustituye a Meteonorm",
        en: "use a site that already has an hourly .MET (sitio meteo_pvgis or convertir_meteo): synthetic_weather does not replace Meteonorm",
    },
}];

/// Fields of a `simular` result that carry PVsyst's own words.
const TEXTOS: [&str; 4] = [
    "detected_error",
    "motivo_ambiguedad",
    "stdout_tail",
    "stderr_tail",
];

/// Everything PVsyst printed for one run, joined.
fn texto_de(salida: &Value) -> String {
    let mut partes: Vec<String> = TEXTOS
        .iter()
        .filter_map(|k| salida.get(*k).and_then(Value::as_str))
        .map(str::to_string)
        .collect();
    let avisos = salida
        .get("avisos_pvsyst")
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
        .filter_map(Value::as_str)
        .map(str::to_string);
    partes.extend(avisos);
    partes.join("\n")
}

/// The known failure in a `simular` result, as `causa — alternativa`.
pub(crate) fn explicar(salida: &Value, idioma: Idioma) -> Option<String> {
    let texto = texto_de(salida);
    FALLOS.iter().find(|f| texto.contains(f.marca)).map(|f| {
        format!(
            "{} — {}",
            f.causa.en_idioma(idioma),
            f.alternativa.en_idioma(idioma)
        )
    })
}

#[cfg(test)]
#[path = "../../tests/unit/easy_mode/fallo_simulacion.rs"]
mod tests;
