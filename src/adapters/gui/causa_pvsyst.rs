//! What PVsyst's own log lines mean for the caller: which notices are noise,
//! which one names the real cause of a failed step, and what that cause calls
//! for. A failed GUI step otherwise reports the symptom (a disabled control, a
//! missing window) with PVsyst's raw log appended.

/// Notices PVsyst writes while reading a file. They appear in every normal run
/// (simulations included) and never change the outcome.
const AVISOS_BENIGNOS: [&str; 2] = ["TEnergyMixSet", "EndTags"];

/// One failure PVsyst logs instead of showing, in the words the caller needs.
struct Causa {
    /// Text PVsyst writes to its log for this failure.
    marca: &'static str,
    /// What actually happened, without PVsyst's internal names.
    causa: &'static str,
    /// The step that gets past it.
    siguiente: &'static str,
}

/// Failures already seen in a live run (see the 2026-11-06 findings report).
const CAUSAS: &[Causa] = &[
    Causa {
        marca: "Empty project name detected in fPrjList",
        causa: "PVsyst opened its project window with no project loaded, so the variant selector stayed disabled and nothing could be read",
        siguiente: "load the project first (proyecto crear, or gui_actuar accion:funcion for proyecto.cargar) and repeat this call",
    },
    Causa {
        marca: "Failed to initialize Meteonorm",
        causa: "PVsyst generates synthetic hourly weather with Meteonorm 9 and this installation has no Meteonorm data (DataMN9/station.db.enc)",
        siguiente: "use a site that already has an hourly .MET, or install Meteonorm; synthetic_weather does not replace it",
    },
];

/// Whether `mensaje` is one of the parse notices PVsyst emits on every load.
pub(crate) fn es_aviso_benigno(mensaje: &str) -> bool {
    AVISOS_BENIGNOS.iter().any(|marca| mensaje.contains(marca))
}

/// `mensajes` without the benign parse notices.
pub(crate) fn filtrar(mensajes: Vec<String>) -> Vec<String> {
    mensajes
        .into_iter()
        .filter(|m| !es_aviso_benigno(m))
        .collect()
}

/// The known failure behind `mensajes`, as `(causa, siguiente)`.
fn causa_conocida(mensajes: &[String]) -> Option<(&'static str, &'static str)> {
    CAUSAS
        .iter()
        .find(|c| mensajes.iter().any(|m| m.contains(c.marca)))
        .map(|c| (c.causa, c.siguiente))
}

/// The text to report for a failed step: `error`, then the real cause when
/// PVsyst logged one, then whatever else PVsyst said that is not benign noise.
pub(crate) fn explicar(error: &str, mensajes: &[String]) -> String {
    let utiles = filtrar(mensajes.to_vec());
    let mut texto = error.to_string();
    if let Some((causa, siguiente)) = causa_conocida(&utiles) {
        texto.push_str(&format!(". Cause: {causa}. Next: {siguiente}"));
    }
    let resto: Vec<&str> = utiles
        .iter()
        .map(String::as_str)
        .filter(|m| !CAUSAS.iter().any(|c| m.contains(c.marca)))
        .collect();
    if !resto.is_empty() {
        texto.push_str(&format!(". PVsyst said: {}", resto.join(" | ")));
    }
    texto
}

#[cfg(test)]
#[path = "../../../tests/unit/adapters/gui/causa_pvsyst.rs"]
mod tests;
